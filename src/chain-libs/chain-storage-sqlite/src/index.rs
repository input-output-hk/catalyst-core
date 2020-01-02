use std::collections::HashMap;

#[derive(Clone)]
pub(crate) struct DbIndex<K, V>(HashMap<K, V>);

impl<K, V> DbIndex<K, V>
where
    K: std::hash::Hash + std::cmp::Eq,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    pub fn add(&mut self, key: K, value: V) {
        self.0.insert(key, value);
    }
}
