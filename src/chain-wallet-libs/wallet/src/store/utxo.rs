use chain_impl_mockchain::{transaction::UtxoPointer, value::Value};
use chain_path_derivation::DerivationPath;
use hdkeygen::Key;
use im_rc::{HashMap, HashSet, OrdMap};
use std::rc::Rc;

type UTxO = Rc<UtxoPointer>;

pub struct UtxoGroup<KEY> {
    by_value: OrdMap<Value, HashSet<UTxO>>,
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
    by_value: OrdMap<Value, HashSet<UTxO>>,

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
            by_value: OrdMap::new(),
            total_value: Value::zero(),
            key: Rc::new(key),
        }
    }

    pub fn key(&self) -> &Rc<KEY> {
        &self.key
    }

    /// utxos already ordered by value
    pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
        self.by_value.values().flatten()
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
        by_value.entry(utxo.value).or_default().insert(utxo);

        Self {
            total_value,
            by_value,
            key,
        }
    }

    fn remove(&self, utxo: &UtxoPointer) -> Option<(UTxO, Self)> {
        use im_rc::ordmap::Entry::*;
        let mut new = self.clone();

        if let Occupied(mut occupied) = new.by_value.entry(utxo.value) {
            if let Some(prev) = occupied.get_mut().remove(utxo) {
                new.total_value = new
                    .total_value
                    .checked_sub(prev.value)
                    .ok()
                    .unwrap_or_else(Value::zero);

                if occupied.get().is_empty() {
                    occupied.remove();
                }

                Some((prev, new))
            } else {
                None
            }
        } else {
            None
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
        self.by_derivation_path.values()
    }

    /// get the UTxO, not grouped, ordered by value
    pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
        self.by_value.values().flatten()
    }

    /// lookup the UTxO group (if any) associated to the given derivation path
    pub fn group(&self, dp: &<KEY as Groupable>::Key) -> Option<&GroupRef<KEY>> {
        self.by_derivation_path.get(dp)
    }

    /// create a new UTxOStore with the added value
    ///
    /// Keeping the previous UtxoStore is useful for quickly switching back
    /// to a previous state is a rollback happened or in case of managing
    /// different forks
    #[must_use = "function does not modify the internal state, the returned value is the new state"]
    pub fn add(&self, utxo: UtxoPointer, key: KEY) -> Self {
        use im_rc::hashmap::Entry::*;

        let mut new = self.clone();
        let utxo = Rc::new(utxo);
        let path = key.group_key();

        new.total_value = new.total_value.saturating_add(utxo.value);
        let group = match new.by_derivation_path.entry(path) {
            Occupied(mut occupied) => {
                let group = occupied.get().clone();
                let group = Rc::new(group.add(Rc::clone(&utxo)));
                *occupied.get_mut() = Rc::clone(&group);
                group
            }
            Vacant(vacant) => {
                let group = UtxoGroup::new(key);
                let group = group.add(Rc::clone(&utxo));

                let group = Rc::new(group);
                vacant.insert(Rc::clone(&group));

                group
            }
        };
        new.by_utxo.insert(Rc::clone(&utxo), group);
        new.by_value.entry(utxo.value).or_default().insert(utxo);

        new
    }

    /// remove the UTxO pointer from the Store.
    ///
    /// returns the updated Store (the returned value is the updated store), `self`
    /// is the previous state. of the store prior removal of the pointer.
    #[must_use = "function does not modify the internal state, the returned value is the new state"]
    pub fn remove(&self, utxo: &UtxoPointer) -> Option<Self> {
        let mut new = self.clone();

        let group = new.by_utxo.remove(utxo)?;
        let path = group.key.group_key();
        let (utxo, new_group) = new.by_derivation_path.get(&path)?.remove(utxo)?;

        *new.by_derivation_path.get_mut(&path).unwrap() = Rc::new(new_group);

        new.by_value.entry(utxo.value).and_modify(|set| {
            set.remove(&utxo);
        });
        new.total_value = new
            .total_value
            .checked_sub(utxo.value)
            .ok()
            .unwrap_or_else(Value::zero);

        Some(new)
    }

    pub fn get_signing_key(&self, utxo: &UtxoPointer) -> Option<Rc<KEY>> {
        self.by_utxo.get(utxo).map(|group| Rc::clone(&group.key))
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
            by_value: OrdMap::new(),
            total_value: Value::zero(),
        }
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

        fn by_key(key: &MockKey, store: &UtxoStore<MockKey>) -> Vec<UtxoPointer> {
            let utxos_by_key = store
                .group(&key.group_key())
                .unwrap()
                .utxos()
                .map(|utxo| **utxo)
                .collect::<Vec<_>>();

            utxos_by_key
        }

        assert_eq!(store1.utxos().map(|_| 1).sum::<u8>(), 0);
        assert!(store1.group(&key.group_key()).is_none());

        assert_eq!(store2.utxos().map(|_| 1).sum::<u8>(), 1);
        assert_eq!(by_key(&key, &store2).len(), 1);

        assert_eq!(store3.utxos().map(|_| 1).sum::<u8>(), 2);
        assert_eq!(by_key(&key, &store3).len(), 2);

        assert_eq!(store4.utxos().map(|_| 1).sum::<u8>(), 2);
        assert_eq!(by_key(&key, &store4).len(), 2);
    }
}
