use chain_impl_mockchain::{transaction::UtxoPointer, value::Value};
use chain_path_derivation::DerivationPath;
use hdkeygen::Key;
use imhamt::Hamt;
use itertools::Itertools as _;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::rc::Rc;

type UTxO = Rc<UtxoPointer>;

type HashMap<K, V> = Hamt<DefaultHasher, K, V>;

#[derive(Clone)]
struct HashSet<T: Hash + PartialEq + Eq + Clone>(HashMap<T, ()>);

pub struct UtxoGroup<KEY> {
    by_value: HashMap<Value, HashSet<UTxO>>,
    total_value: Value,
    key: Rc<KEY>,
}

type GroupRef<KEY> = Rc<UtxoGroup<KEY>>;

/// A UTxO store that can be cheaply updated/cloned
///
/// The data structure of the store allows for sharing states between multiple
/// reality of the blockchain. Following different branches if needed.
///
/// The UTxO store, if used for a bip32 scheme based wallet, needs to be for
/// one given wallet. I.e. multiple wallets will need different Store.
///
/// ## performance considerations
///
/// The Store has been optimized to allow following multiple branches of a
/// blockchain (multiple reality of what the blockchain's state can be).
/// It uses a HAMT (Hash Array Mapped tries) to allow for the internal
/// collections to share the common states. So having multiple derivation
/// of a given UtxoStore takes only as much memory as the difference of
/// state between 2 `UtxoStore`.
///
/// The internal organization of the store is optimal for small to medium size
/// UTxO Stores. For larger UTxO stores it is more interesting to use a different
/// data structure, tuned for the need.
///
pub struct UtxoStore<KEY: Groupable> {
    by_utxo: HashMap<UTxO, GroupRef<KEY>>,
    by_derivation_path: HashMap<<KEY as Groupable>::Key, GroupRef<KEY>>,
    by_value: HashMap<Value, HashSet<UTxO>>,

    total_value: Value,
}

/// Define the way the utxos should be grouped, as secret keys may not be
/// hashable/comparable by themselves.
pub trait Groupable {
    type Key: std::hash::Hash + Eq + Clone;

    fn group_key(&self) -> Self::Key;
}

impl<KIND, SCHEME> Groupable for Key<KIND, SCHEME> {
    type Key = DerivationPath<SCHEME>;

    fn group_key(&self) -> Self::Key {
        self.path().clone()
    }
}

impl Groupable for chain_crypto::SecretKey<chain_crypto::Ed25519Extended> {
    type Key = chain_crypto::PublicKey<chain_crypto::Ed25519>;

    fn group_key(&self) -> Self::Key {
        self.to_public()
    }
}

impl<KEY> UtxoGroup<KEY> {
    fn new(key: KEY) -> Self {
        Self {
            by_value: HashMap::new(),
            total_value: Value::zero(),
            key: Rc::new(key),
        }
    }

    pub fn key(&self) -> &Rc<KEY> {
        &self.key
    }

    /// utxos already ordered by value
    pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
        self.by_value
            .iter()
            .sorted_by_key(|x| x.0)
            .map(|(_k, set)| set.iter())
            .flatten()
    }

    /// total value of the given group
    pub fn total_value(&self) -> Value {
        self.total_value
    }

    fn add(&self, utxo: UTxO) -> Self {
        let Self {
            total_value,
            mut by_value,
            key,
        } = self.clone();

        let total_value = total_value.saturating_add(utxo.value);

        by_value = by_value.insert_or_update_simple(
            utxo.value,
            {
                let set = HashSet::new();
                let set = set.insert(utxo.clone());
                set
            },
            |old| Some(old.insert(utxo.clone())),
        );

        Self {
            total_value,
            by_value,
            key,
        }
    }

    fn remove(&self, utxo: &UtxoPointer) -> Self {
        // let mut new = self.clone();

        let Self {
            by_value,
            mut total_value,
            key,
        } = self.clone();

        let by_value = by_value
            .update::<_, std::convert::Infallible>(&utxo.value, |set| {
                let new_set = set.remove(&Rc::new(utxo.clone()));

                total_value = total_value
                    .checked_sub(utxo.value)
                    .ok()
                    .unwrap_or_else(Value::zero);

                Ok(Some(new_set))
            })
            .unwrap();

        Self {
            by_value,
            total_value,
            key,
        }
    }
}

impl<KEY: Groupable> UtxoStore<KEY> {
    pub fn new() -> Self {
        Self::default()
    }

    /// get the current total value
    ///
    /// this also include all the unconfirmed transactions
    pub fn total_value(&self) -> Value {
        self.total_value
    }

    /// get an iterator over the UTxO grouped by the same key
    ///
    /// this allows optimizing the search of inputs and to favors using
    /// inputs of the same key to preserve privacy
    pub fn groups(&self) -> impl Iterator<Item = &GroupRef<KEY>> {
        self.by_derivation_path.iter().map(|(_, v)| v)
    }

    /// get the UTxO, not grouped, ordered by value
    pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
        self.by_value
            .iter()
            .sorted_by_key(|x| x.0)
            .map(|(_, set)| set.iter())
            .flatten()
    }

    /// lookup the UTxO group (if any) associated to the given derivation path
    pub fn group(&self, dp: &<KEY as Groupable>::Key) -> Option<&GroupRef<KEY>> {
        self.by_derivation_path.lookup(dp)
    }

    /// create a new UTxOStore with the added value
    ///
    /// Keeping the previous UtxoStore is useful for quickly switching back
    /// to a previous state is a rollback happened or in case of managing
    /// different forks
    #[must_use = "function does not modify the internal state, the returned value is the new state"]
    pub fn add(&self, utxo: UtxoPointer, key: KEY) -> Self {
        let mut new = self.clone();
        let utxo = Rc::new(utxo);
        let path = key.group_key();

        new.total_value = new.total_value.saturating_add(utxo.value);

        new.by_derivation_path = new.by_derivation_path.insert_or_update_simple(
            path.clone(),
            {
                let group = UtxoGroup::new(key);
                let group = group.add(Rc::clone(&utxo));

                Rc::new(group)
            },
            |group| Some(Rc::new(group.add(Rc::clone(&utxo)))),
        );

        // XXX: could be optimized with a mut Option
        let group = new.by_derivation_path.lookup(&path).unwrap().clone();

        new.by_utxo = new
            .by_utxo
            .insert(Rc::clone(&utxo), group)
            .unwrap_or(new.by_utxo);

        new.by_value = new.by_value.insert_or_update_simple(
            utxo.value,
            HashSet::new().insert(utxo.clone()),
            |old| Some(old.insert(utxo.clone())),
        );

        new
    }

    /// remove the UTxO pointer from the Store.
    ///
    /// returns the updated Store (the returned value is the updated store), `self`
    /// is the previous state. of the store prior removal of the pointer.
    #[must_use = "function does not modify the internal state, the returned value is the new state"]
    pub fn remove(&self, utxo: &UtxoPointer) -> Option<Self> {
        let mut new = self.clone();

        let utxo = Rc::new(*utxo);

        let group = new.by_utxo.lookup(&utxo)?;
        let path = group.key.group_key();

        new.by_utxo.remove(&Rc::new(*utxo)).ok()?;

        new.by_derivation_path = new
            .by_derivation_path
            .update::<_, std::convert::Infallible>(&path, |group| {
                Ok(Some(Rc::new(group.remove(&utxo))))
            })
            .unwrap();

        // new.by_value.entry(utxo.value).and_modify(|set| {
        //     *set = set.remove(&utxo);
        // });

        new.by_value = new
            .by_value
            .update::<_, std::convert::Infallible>(&utxo.value, |set| Ok(Some(set.remove(&utxo))))
            .unwrap();

        new.total_value = new
            .total_value
            .checked_sub(utxo.value)
            .ok()
            .unwrap_or_else(Value::zero);

        Some(new)
    }

    pub fn get_signing_key(&self, utxo: &UtxoPointer) -> Option<Rc<KEY>> {
        self.by_utxo
            .lookup(&Rc::new(utxo.clone()))
            .map(|group| Rc::clone(&group.key))
    }
}

impl<KEY> Clone for UtxoGroup<KEY> {
    fn clone(&self) -> Self {
        Self {
            by_value: self.by_value.clone(),
            total_value: self.total_value,
            key: Rc::clone(&self.key),
        }
    }
}

impl<KEY: Groupable> Clone for UtxoStore<KEY> {
    fn clone(&self) -> Self {
        Self {
            by_utxo: self.by_utxo.clone(),
            by_derivation_path: self.by_derivation_path.clone(),
            by_value: self.by_value.clone(),
            total_value: self.total_value,
        }
    }
}

impl<KEY: Groupable> Default for UtxoStore<KEY> {
    fn default() -> Self {
        Self {
            by_utxo: HashMap::new(),
            by_derivation_path: HashMap::new(),
            by_value: HashMap::new(),
            total_value: Value::zero(),
        }
    }
}

impl<T: Hash + PartialEq + Eq + Clone> HashSet<T> {
    fn new() -> Self {
        Self(HashMap::new())
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter().map(|(k, _)| k)
    }

    #[must_use = "this structure is immutable, the new one is returned"]
    fn insert(&self, element: T) -> Self {
        Self(self.0.insert(element, ()).unwrap_or(self.0.clone()))
    }

    #[must_use = "this structure is immutable, the new one is returned"]
    fn remove(&self, element: &T) -> Self {
        Self(self.0.remove(element).unwrap_or(self.0.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone)]
    struct MockKey(u8);

    #[derive(Hash, PartialEq, Eq, Copy, Clone)]
    struct MockGroupKey(u8);

    impl Groupable for MockKey {
        type Key = MockGroupKey;

        fn group_key(&self) -> Self::Key {
            MockGroupKey(self.0)
        }
    }

    fn by_key(key: &MockKey, store: &UtxoStore<MockKey>) -> Vec<UtxoPointer> {
        let utxos_by_key = store
            .group(&key.group_key())
            .unwrap()
            .utxos()
            .map(|utxo| **utxo)
            .collect::<Vec<_>>();

        utxos_by_key
    }

    #[test]
    fn test_add_does_not_share_state() {
        use chain_impl_mockchain::key::Hash;
        let key = MockKey(0);
        let store1 = UtxoStore::<MockKey>::new();

        let store2 = store1.add(
            UtxoPointer {
                transaction_id: Hash::from_bytes([0u8; 32]),
                output_index: 0u8,
                value: Value(100),
            },
            key,
        );

        let store3 = store2.add(
            UtxoPointer {
                transaction_id: Hash::from_bytes([1u8; 32]),
                output_index: 1u8,
                value: Value(100),
            },
            key,
        );

        let store4 = store2.add(
            UtxoPointer {
                transaction_id: Hash::from_bytes([2u8; 32]),
                output_index: 2u8,
                value: Value(1000),
            },
            key,
        );

        assert_eq!(store1.utxos().map(|_| 1).sum::<u8>(), 0);
        assert!(store1.group(&key.group_key()).is_none());

        assert_eq!(store2.utxos().map(|_| 1).sum::<u8>(), 1);
        assert_eq!(by_key(&key, &store2).len(), 1);

        assert_eq!(store3.utxos().map(|_| 1).sum::<u8>(), 2);
        assert_eq!(by_key(&key, &store3).len(), 2);

        assert_eq!(store4.utxos().map(|_| 1).sum::<u8>(), 2);
        assert_eq!(by_key(&key, &store4).len(), 2);
    }

    #[test]
    fn test_remove_does_not_share_state() {
        use chain_impl_mockchain::key::Hash;
        let key = MockKey(0);
        let store1 = UtxoStore::<MockKey>::new();

        let utxo1 = UtxoPointer {
            transaction_id: Hash::from_bytes([0u8; 32]),
            output_index: 0u8,
            value: Value(100),
        };

        let utxo2 = UtxoPointer {
            transaction_id: Hash::from_bytes([1u8; 32]),
            output_index: 1u8,
            value: Value(100),
        };

        let utxo3 = UtxoPointer {
            transaction_id: Hash::from_bytes([2u8; 32]),
            output_index: 2u8,
            value: Value(1000),
        };

        let store2 = store1.add(utxo1, key);

        let store3 = store2.add(utxo2, key);

        let store4 = store3.add(utxo3, key);

        assert_eq!(store4.utxos().map(|_| 1).sum::<u8>(), 3);
        assert_eq!(by_key(&key, &store4).len(), 3);

        let minus_2 = store3.remove(&utxo2).unwrap();

        assert!(!minus_2.utxos().any(|utxo| utxo.as_ref() == &utxo2));

        assert_eq!(store4.utxos().map(|_| 1).sum::<u8>(), 3);
        assert_eq!(by_key(&key, &store4).len(), 3);
    }

    #[test]
    fn test_utxos_are_sorted() {
        use chain_impl_mockchain::key::Hash;
        let key = MockKey(0);
        let store1 = UtxoStore::<MockKey>::new();

        let utxo1 = UtxoPointer {
            transaction_id: Hash::from_bytes([0u8; 32]),
            output_index: 0u8,
            value: Value(100),
        };

        let utxo2 = UtxoPointer {
            transaction_id: Hash::from_bytes([1u8; 32]),
            output_index: 1u8,
            value: Value(300),
        };

        let utxo3 = UtxoPointer {
            transaction_id: Hash::from_bytes([2u8; 32]),
            output_index: 2u8,
            value: Value(200),
        };

        let store2 = store1.add(utxo1, key);
        let store3 = store2.add(utxo2, key);
        let store4 = store3.add(utxo3, key);

        itertools::assert_equal(store4.utxos().map(|utxo| utxo.value.0), vec![100, 200, 300]);
    }
}
