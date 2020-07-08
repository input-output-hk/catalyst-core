use std::{
    borrow::Borrow,
    collections::HashMap,
    hash::{Hash, Hasher},
    iter::FusedIterator,
};

struct State<K, S> {
    key: K,
    state: S,
    confirmed: bool,
    prev: *mut State<K, S>,
    next: *mut State<K, S>,
}

#[doc(hidden)]
pub struct KeyRef<K>(*const K);

pub struct StateIter<'a, K, S> {
    forward: *mut State<K, S>,
    backward: *mut State<K, S>,
    len: usize,
    _anchor: std::marker::PhantomData<&'a (K, S)>,
}

pub struct Multiverse<K, S> {
    map: HashMap<KeyRef<K>, Box<State<K, S>>>,

    head: *mut State<K, S>,
    tail: *mut State<K, S>,
}

impl<K, S> State<K, S> {
    fn new(key: K, state: S, confirmed: bool) -> Self {
        Self {
            key,
            state,
            confirmed,
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }
    }

    fn confirmed(&self) -> bool {
        self.confirmed
    }

    fn confirm(&mut self) {
        self.confirmed = true
    }
}

impl<K, S> Multiverse<K, S>
where
    K: Hash + Eq,
{
    /// create a new Multiverse with the given initial state
    ///
    /// by default this state is always assumed confirmed
    pub fn new(key: K, state: S) -> Self {
        let mut state = Box::new(State::new(key, state, true));
        let key_ref = KeyRef(&state.key);
        let head: *mut State<K, S> = &mut *state;
        let tail: *mut State<K, S> = &mut *state;

        let mut map = HashMap::with_capacity(12);
        map.insert(key_ref, state);

        Self { map, head, tail }
    }

    /// check wether the given state associate to this key is present
    /// in the Multiverse
    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.contains_key(key)
    }

    /// get the underlying State associated to the given key
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&S>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.get(key).map(|s| &s.state)
    }

    /// push a new **unconfirmed** state in the Multiverse
    pub fn push(&mut self, key: K, state: S) {
        let mut state = Box::new(State::new(key, state, false));
        let key_ref = KeyRef(&state.key);

        state.prev = self.tail;
        unsafe { (*self.tail).next = state.as_mut() };
        self.tail = state.as_mut();

        assert!(self.map.insert(key_ref, state).is_none());
    }

    pub fn confirm<Q: ?Sized>(&mut self, key: &Q)
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq,
    {
        if let Some(state) = self.map.get_mut(key) {
            let state = &mut (*state) as &mut State<K, S>;
            state.confirm();
        }

        while self.pop_legacy_confirmed() {}
    }

    fn pop_legacy_confirmed(&mut self) -> bool {
        let current = unsafe { &mut (*self.head) as &mut State<K, S> };
        debug_assert!(current.confirmed());

        if let Some(next) = unsafe { current.next.as_mut() } {
            if next.confirmed() {
                self.map.remove(&current.key);

                self.head = current.next;
                next.prev = std::ptr::null_mut();

                return true;
            }
        }

        false
    }
}

impl<K, S> Multiverse<K, S> {
    /// get the number of states in the Multiverse
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// always return false
    pub fn is_empty(&self) -> bool {
        debug_assert!(!self.map.is_empty());
        self.map.is_empty()
    }

    /// iterate through the states from the confirmed one up to the most
    /// recent one.
    ///
    /// there is always at least one element in the iterator (the confirmed one).
    pub fn iter(&self) -> StateIter<'_, K, S> {
        StateIter {
            forward: self.head,
            backward: self.tail,
            len: self.len(),
            _anchor: std::marker::PhantomData,
        }
    }

    /// access the confirmed state of the store verse
    pub fn confirmed_state(&self) -> (&K, &S) {
        if let Some(state) = unsafe { self.head.as_ref() } {
            debug_assert!(state.confirmed());
            (&state.key, &state.state)
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }

    /// get the most recent un confirmed state
    pub fn last_unconfirmed_state(&self) -> Option<(&K, &S)> {
        if let Some(state) = unsafe { self.tail.as_ref() } {
            if state.confirmed() && self.tail == self.head {
                None
            } else {
                Some((&state.key, &state.state))
            }
        } else {
            unsafe { std::hint::unreachable_unchecked() }
        }
    }
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.0).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &KeyRef<K>) -> bool {
        unsafe { (*self.0).eq(&*other.0) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

#[cfg(not(feature = "nightly"))]
impl<K> Borrow<K> for KeyRef<K> {
    fn borrow(&self) -> &K {
        unsafe { &*self.0 }
    }
}

impl<'a, K, S> IntoIterator for &'a Multiverse<K, S> {
    type Item = (&'a K, &'a S);
    type IntoIter = StateIter<'a, K, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, S> Iterator for StateIter<'a, K, S> {
    type Item = (&'a K, &'a S);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.forward).key as &K };
        let state = unsafe { &(*self.forward).state as &S };

        self.len -= 1;
        self.forward = unsafe { (*self.forward).next };

        Some((key, state))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize {
        self.len
    }
}

impl<'a, K, S> DoubleEndedIterator for StateIter<'a, K, S> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*self.backward).key as &K };
        let state = unsafe { &(*self.backward).state as &S };

        self.len -= 1;
        self.backward = unsafe { (*self.backward).prev };

        Some((key, state))
    }
}

impl<'a, K, S> FusedIterator for StateIter<'a, K, S> {}
impl<'a, K, S> ExactSizeIterator for StateIter<'a, K, S> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_iterator() {
        let mut multiverse = Multiverse::new(0u8, ());
        multiverse.push(1, ());
        multiverse.push(2, ());
        multiverse.push(3, ());
        multiverse.push(4, ());
        multiverse.push(5, ());

        assert_eq!(multiverse.len(), 6, "invalid length");
        assert!(!multiverse.is_empty());

        let mut iter = multiverse.iter();

        assert_eq!(Some((&0, &())), iter.next());
        assert_eq!(Some((&1, &())), iter.next());
        assert_eq!(Some((&2, &())), iter.next());
        assert_eq!(Some((&3, &())), iter.next());
        assert_eq!(Some((&4, &())), iter.next());
        assert_eq!(Some((&5, &())), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn backward_iterator() {
        let mut multiverse = Multiverse::new(0u8, ());
        multiverse.push(1, ());
        multiverse.push(2, ());
        multiverse.push(3, ());
        multiverse.push(4, ());
        multiverse.push(5, ());

        assert_eq!(multiverse.len(), 6, "invalid length");
        assert!(!multiverse.is_empty());

        let mut iter = multiverse.iter();

        assert_eq!(Some((&5, &())), iter.next_back());
        assert_eq!(Some((&4, &())), iter.next_back());
        assert_eq!(Some((&3, &())), iter.next_back());
        assert_eq!(Some((&2, &())), iter.next_back());
        assert_eq!(Some((&1, &())), iter.next_back());
        assert_eq!(Some((&0, &())), iter.next_back());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn double_ended_iterator() {
        let mut multiverse = Multiverse::new(0u8, ());
        multiverse.push(1, ());
        multiverse.push(2, ());
        multiverse.push(3, ());
        multiverse.push(4, ());
        multiverse.push(5, ());

        assert_eq!(multiverse.len(), 6, "invalid length");
        assert!(!multiverse.is_empty());

        let mut iter = multiverse.iter();

        assert_eq!(Some((&0, &())), iter.next());
        assert_eq!(Some((&5, &())), iter.next_back());
        assert_eq!(Some((&4, &())), iter.next_back());
        assert_eq!(Some((&1, &())), iter.next());
        assert_eq!(Some((&2, &())), iter.next());
        assert_eq!(Some((&3, &())), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn confirmed_state() {
        let mut multiverse = Multiverse::new(0u8, ());
        assert_eq!((&0, &()), multiverse.confirmed_state());
        assert_eq!(None, multiverse.last_unconfirmed_state());

        multiverse.push(1, ());
        assert_eq!((&0, &()), multiverse.confirmed_state());
        assert_eq!(Some((&1, &())), multiverse.last_unconfirmed_state());

        multiverse.push(2, ());
        multiverse.push(3, ());
        multiverse.push(4, ());
        assert_eq!((&0, &()), multiverse.confirmed_state());
        assert_eq!(Some((&4, &())), multiverse.last_unconfirmed_state());

        multiverse.confirm(&1);
        assert_eq!((&1, &()), multiverse.confirmed_state());
        assert_eq!(Some((&4, &())), multiverse.last_unconfirmed_state());

        multiverse.confirm(&4);
        assert_eq!((&1, &()), multiverse.confirmed_state());
        assert_eq!(Some((&4, &())), multiverse.last_unconfirmed_state());

        multiverse.confirm(&3);
        multiverse.confirm(&2);
        assert_eq!((&4, &()), multiverse.confirmed_state());
        assert_eq!(None, multiverse.last_unconfirmed_state());
    }
}
