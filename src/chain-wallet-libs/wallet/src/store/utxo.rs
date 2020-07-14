use chain_impl_mockchain::{transaction::UtxoPointer, value::Value};
use chain_path_derivation::DerivationPath;
use ed25519_bip32::XPrv;
use hdkeygen::Key;
use im_rc::{HashMap, HashSet, OrdMap};
use std::{cell::RefCell, rc::Rc};

type UTxO = Rc<UtxoPointer>;

pub struct UtxoGroup<K> {
    by_value: OrdMap<Value, HashSet<UTxO>>,
    total_value: Value,
    key: Rc<Key<XPrv, K>>,
}

type GroupRef<K> = Rc<RefCell<UtxoGroup<K>>>;

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
pub struct UtxoStore<K> {
    by_utxo: HashMap<UTxO, GroupRef<K>>,
    by_derivation_path: HashMap<DerivationPath<K>, GroupRef<K>>,
    by_value: OrdMap<Value, HashSet<UTxO>>,

    total_value: Value,
}

impl<K> UtxoGroup<K> {
    fn new(key: Key<XPrv, K>) -> Self {
        Self {
            by_value: OrdMap::new(),
            total_value: Value::zero(),
            key: Rc::new(key),
        }
    }

    pub fn key(&self) -> &Rc<Key<XPrv, K>> {
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

impl<K> UtxoStore<K> {
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
    pub fn groups(&self) -> impl Iterator<Item = std::cell::Ref<'_, UtxoGroup<K>>> {
        self.by_derivation_path.values().map(|r| r.borrow())
    }

    /// get the UTxO, not grouped, ordered by value
    pub fn utxos(&self) -> impl Iterator<Item = &UTxO> {
        self.by_value.values().flatten()
    }

    /// lookup the UTxO group (if any) associated to the given derivation path
    pub fn group(&self, dp: &DerivationPath<K>) -> Option<&GroupRef<K>> {
        self.by_derivation_path.get(dp)
    }

    /// create a new UTxOStore with the added value
    ///
    /// Keeping the previous UtxoStore is useful for quickly switching back
    /// to a previous state is a rollback happened or in case of managing
    /// different forks
    #[must_use = "function does not modify the internal state, the returned value is the new state"]
    pub fn add(&self, utxo: UtxoPointer, key: Key<XPrv, K>) -> Self {
        use im_rc::hashmap::Entry::*;

        let mut new = self.clone();
        let utxo = Rc::new(utxo);
        let path = key.path().clone();

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
        let path = group.borrow().key.path().clone();
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
}

impl<K> Clone for UtxoGroup<K> {
    fn clone(&self) -> Self {
        Self {
            by_value: self.by_value.clone(),
            total_value: self.total_value,
            key: Rc::clone(&self.key),
        }
    }
}

impl<K> Clone for UtxoStore<K> {
    fn clone(&self) -> Self {
        Self {
            by_utxo: self.by_utxo.clone(),
            by_derivation_path: self.by_derivation_path.clone(),
            by_value: self.by_value.clone(),
            total_value: self.total_value,
        }
    }
}

impl<K> Default for UtxoStore<K> {
    fn default() -> Self {
        Self {
            by_utxo: HashMap::new(),
            by_derivation_path: HashMap::new(),
            by_value: OrdMap::new(),
            total_value: Value::zero(),
        }
    }
}
