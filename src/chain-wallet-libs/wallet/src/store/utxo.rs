use chain_impl_mockchain::{transaction::UtxoPointer, value::Value};
use chain_path_derivation::DerivationPath;
use hdkeygen::Key;
use im_rc::{HashMap, HashSet, OrdMap};
use std::{cell::RefCell, rc::Rc};

type UTxO = Rc<UtxoPointer>;

pub struct UtxoGroup<KEY> {
    by_value: OrdMap<Value, HashSet<UTxO>>,
    total_value: Value,
    key: Rc<KEY>,
}

type GroupRef<KEY> = Rc<RefCell<UtxoGroup<KEY>>>;

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

    fn add(&mut self, utxo: UTxO) {
        self.total_value = self.total_value.saturating_add(utxo.value);
        self.by_value.entry(utxo.value).or_default().insert(utxo);
    }

    fn remove(&mut self, utxo: &UtxoPointer) -> Option<UTxO> {
        use im_rc::ordmap::Entry::*;

        if let Occupied(mut occupied) = self.by_value.entry(utxo.value) {
            if let Some(prev) = occupied.get_mut().remove(utxo) {
                self.total_value = self
                    .total_value
                    .checked_sub(prev.value)
                    .ok()
                    .unwrap_or_else(Value::zero);

                if occupied.get().is_empty() {
                    occupied.remove();
                }

                Some(prev)
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
    pub fn groups(&self) -> impl Iterator<Item = std::cell::Ref<'_, UtxoGroup<KEY>>> {
        self.by_derivation_path.values().map(|r| r.borrow())
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
            Occupied(occupied) => {
                let group = occupied.get().clone();
                group.borrow_mut().add(Rc::clone(&utxo));
                group
            }
            Vacant(vacant) => {
                let group = Rc::new(RefCell::new(UtxoGroup::new(key)));
                vacant
                    .insert(group.clone())
                    .borrow_mut()
                    .add(Rc::clone(&utxo));
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
        let path = group.borrow().key.group_key();
        let utxo = new
            .by_derivation_path
            .get_mut(&path)?
            .borrow_mut()
            .remove(utxo)?;
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
        self.by_utxo
            .get(utxo)
            .map(|group| Rc::clone(&group.borrow().key))
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
    /*     use super::{UTxO, UtxoPointer};
       use quickcheck_macros::*;
       use std::collections::HashSet;

       struct StoreModel {
           utxos: HashSet<UTxO>,
       }

       struct GroupModel {}

       type Key = u32;
       type Scheme = u32;

       impl StoreModel {
           pub fn new() -> Self {
               Self {}
           }

           pub fn total_value(&self) -> Value {
               self.total_value
           }

           pub fn groups(&self) -> impl Iterator<Item = GroupModel> {
               todo!()
           }

           pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
               todo!()
           }

           pub fn group(&self, dp: &Scheme) -> Option<GroupModel> {
               todo!()
           }

           pub fn add(&self, utxo: UtxoPointer, key: Key) -> Self {
               todo!()
           }

           pub fn remove(&self, utxo: &UtxoPointer) -> Option<Self> {
               todo!()
           }

           pub fn get_signing_key(&self, utxo: &UtxoPointer) -> Option<Key> {
               todo!()
           }
       }

       #[quickcheck]
       fn prop(xs: Vec<u32>) -> bool {
           todo!()
       }
    */
}
