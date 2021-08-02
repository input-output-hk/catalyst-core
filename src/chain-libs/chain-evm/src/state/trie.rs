use imhamt::{Hamt, RemoveError};

use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::Hash;

/// An immutable structure to represent any of EVM tries.
#[derive(Clone, Default)]
pub struct Trie<K: Clone + Hash + Eq, V: Clone>(Hamt<DefaultHasher, K, V>);

impl<K: Clone + Hash + Eq, V: Clone> Trie<K, V> {
    pub fn new() -> Self {
        Self(Hamt::new())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.lookup(key)
    }

    /// Check if this trie contains a given key.
    pub fn contains(&self, key: &K) -> bool {
        self.0.contains_key(key)
    }

    /// Put a value into a trie replacing an exisisting value if there was any.
    pub fn put(self, key: K, value: V) -> Self {
        // using two branches instead of `Hamt::insert_or_update` to avoid unnecessary cloning
        let new_state = if self.0.contains_key(&key) {
            self.0
                .update(&key, |_| Ok::<_, Infallible>(Some(value)))
                .expect("we already checked that the key is present")
        } else {
            self.0
                .insert(key, value)
                .expect("we already checked that the key does not exist")
        };
        Self(new_state)
    }

    /// Remove a value from a trie.
    pub fn remove(self, key: &K) -> Self {
        match self.0.remove(key) {
            Ok(new_state) => Self(new_state),
            Err(RemoveError::KeyNotFound) => self,
            Err(RemoveError::ValueNotMatching) => {
                unreachable!("this error should never occur: we are not matching the removed value")
            }
        }
    }

    /// Put a value into a trie, replacing an existing value if there was any.
    /// This method is useful when you have a mutable reference to the Trie.
    ///
    /// This method is similar to `Trie::put`, except that it uses `&mut self` to
    /// modify the inner state.
    pub fn insert_or_update(&mut self, key: K, value: V) {
        // using two branches instead of `Hamt::insert_or_update` to avoid unnecessary cloning
        let new_state = if self.0.contains_key(&key) {
            self.0
                .update(&key, |_| Ok::<_, Infallible>(Some(value)))
                .expect("we already checked that the key is present")
        } else {
            self.0
                .insert(key, value)
                .expect("we already checked that the key does not exist")
        };
        *self = Self(new_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use test_strategy::proptest;

    #[proptest]
    fn put_get_remove(key: u8, value1: u8, value2: u8) {
        let storage = Trie::new();

        // first write
        let storage_new1 = storage.put(key.clone(), value1.clone());
        prop_assert_eq!(Some(&value1), storage_new1.get(&key));

        // overwriting value
        let storage_new2 = storage_new1.put(key.clone(), value2.clone());
        prop_assert_eq!(Some(&value2), storage_new2.get(&key));

        // removing value
        let storage_new3 = storage_new2.remove(&key);
        prop_assert_eq!(None, storage_new3.get(&key));

        // removing non-existent value should not error
        storage_new3.remove(&key);
    }

    #[proptest]
    fn insert_or_update(key: u8, value1: u8, value2: u8) {
        let mut storage = Trie::<u8, u8>::new();

        prop_assert_eq!(None, storage.get(&key));

        storage.insert_or_update(key, value1);
        prop_assert_eq!(Some(&value1), storage.get(&key));

        storage.insert_or_update(key, value2);
        prop_assert_eq!(Some(&value2), storage.get(&key))
    }
}
