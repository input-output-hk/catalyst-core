use imhamt::{Hamt, HamtIter, RemoveError};

use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use std::hash::Hash;

/// An immutable structure to represent any of EVM tries.
#[derive(Clone, PartialEq, Eq)]
pub struct Trie<K: Eq + Hash, V>(Hamt<DefaultHasher, K, V>);

impl<K: Hash + Eq, V> Default for Trie<K, V> {
    fn default() -> Self {
        Trie(Default::default())
    }
}

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

    /// Update or put the element at the key K
    ///
    /// If the element is not present, then V is added, otherwise the closure F is applied
    /// to the found element. If the closure returns None, then the key is deleted
    pub fn put_or_update<F>(self, key: K, value: V, f: F) -> Self
    where
        F: FnOnce(&V) -> Option<V>,
    {
        let f = |val: &V| -> Result<Option<V>, std::fmt::Error> { Ok(f(val)) };
        let new_state = self.0.insert_or_update(key, value, f).expect("");
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

    pub fn iter(&self) -> HamtIter<'_, K, V> {
        self.0.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a, K: Hash + Eq, V> IntoIterator for &'a Trie<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = HamtIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;
    use test_strategy::proptest;

    #[proptest]
    fn put_get_remove_test(key: u8, value1: u8, value2: u8) {
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
    fn put_or_update_test(key: u8, value1: u8, value2: u8) {
        let storage = Trie::new();

        let vec1 = vec![value1.clone()];
        let vec2 = vec![value1.clone(), value2.clone()];

        // first write
        let storage_new1 = storage.put_or_update(
            key.clone(),
            vec1.clone(),
            |val: &Vec<u8>| -> Option<Vec<u8>> {
                let mut val = val.clone();
                val.push(value1);
                Some(val)
            },
        );
        prop_assert_eq!(Some(&vec1), storage_new1.get(&key));

        // update value
        let storage_new2 =
            storage_new1.put_or_update(key, vec2.clone(), |val: &Vec<u8>| -> Option<Vec<u8>> {
                let mut val = val.clone();
                val.push(value2);
                Some(val)
            });
        prop_assert_eq!(Some(&vec2), storage_new2.get(&key));
    }
}
