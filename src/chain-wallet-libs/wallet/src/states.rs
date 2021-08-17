use std::{borrow::Borrow, hash::Hash};

use hashlink::LinkedHashMap;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum Status {
    Confirmed,
    Pending,
}

#[derive(Debug)]
pub struct StateRef<K, S> {
    key: K,
    state: S,
    status: Status,
}

pub struct States<K, S> {
    states: LinkedHashMap<K, StateRef<K, S>>,
}

impl<K: std::fmt::Debug, S: std::fmt::Debug> std::fmt::Debug for States<K, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<K, S> StateRef<K, S> {
    fn new(key: K, state: S, status: Status) -> Self {
        Self { key, state, status }
    }

    pub fn is_confirmed(&self) -> bool {
        matches!(self.status, Status::Confirmed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self.status, Status::Pending)
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn key(&self) -> &K {
        &self.key
    }

    fn confirm(&mut self) {
        self.status = Status::Confirmed
    }
}

impl<K, S> States<K, S>
where
    K: Hash + Eq + Clone,
{
    /// create a new States with the given initial state
    ///
    /// by default this state is always assumed confirmed
    pub fn new(key: K, state: S) -> Self {
        let state = StateRef::new(key.clone(), state, Status::Confirmed);
        let mut states = LinkedHashMap::new();
        states.insert(key, state);

        Self { states }
    }

    /// check wether the given state associate to this key is present
    /// in the States
    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.states.contains_key(key)
    }

    /// push a new **unconfirmed** state in the States
    pub fn push(&mut self, key: K, state: S) {
        let state = StateRef::new(key.clone(), state, Status::Pending);

        assert!(self.states.insert(key, state).is_none());
    }

    pub fn confirm<Q: ?Sized>(&mut self, key: &Q)
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        if let Some(state) = self.states.get_mut(key) {
            state.confirm();
        }

        self.pop_old_confirmed_states()
    }

    fn pop_old_confirmed_states(&mut self) {
        let mut stack = std::collections::VecDeque::with_capacity(2);

        loop {
            if stack.len() == 2 {
                stack.pop_front();
            }

            if let Some((key, state)) = self.states.pop_front() {
                let is_pending = state.is_pending();

                stack.push_back((key, state));

                if is_pending {
                    break;
                }
            } else {
                break;
            }
        }

        for (key, state) in stack.drain(..).rev() {
            self.states.insert(key.clone(), state);
            self.states.to_front(&key);
        }

        debug_assert!(self.states.front().unwrap().1.is_confirmed());
    }
}

impl<K, S> States<K, S> {
    /// iterate through the states from the confirmed one up to the most
    /// recent one.
    ///
    /// there is always at least one element in the iterator (the confirmed one).
    pub fn iter(&self) -> impl Iterator<Item = (&K, &StateRef<K, S>)> {
        self.states.iter()
    }

    pub fn unconfirmed_states(&self) -> impl Iterator<Item = &StateRef<K, S>> {
        self.states.values().filter(|s| s.is_pending())
    }

    /// access the confirmed state of the store verse
    pub fn confirmed_state(&self) -> &StateRef<K, S> {
        self.states.front().map(|(_, v)| v).unwrap()
    }

    /// get the last state of the store
    pub fn last_state(&self) -> &StateRef<K, S> {
        self.states.back().unwrap().1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for StateRef<u8, ()> {
        fn eq(&self, other: &Self) -> bool {
            (self.key, self.status).eq(&(other.key, other.status))
        }
    }

    impl PartialOrd for StateRef<u8, ()> {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            (self.key, self.status).partial_cmp(&(other.key, other.status))
        }
    }

    impl StateRef<u8, ()> {
        fn new_confirmed(key: u8) -> Self {
            Self {
                key,
                state: (),
                status: Status::Confirmed,
            }
        }

        fn new_pending(key: u8) -> Self {
            Self {
                key,
                state: (),
                status: Status::Pending,
            }
        }
    }

    #[test]
    fn confirmed_state() {
        let mut multiverse = States::new(0u8, ());
        assert_eq!(&StateRef::new_confirmed(0), multiverse.confirmed_state());

        assert_eq!(&StateRef::new_confirmed(0), multiverse.last_state());

        multiverse.push(1, ());
        assert_eq!(&StateRef::new_confirmed(0), multiverse.confirmed_state());
        assert_eq!(&StateRef::new_pending(1), multiverse.last_state());

        multiverse.push(2, ());
        multiverse.push(3, ());
        multiverse.push(4, ());
        assert_eq!(&StateRef::new_confirmed(0), multiverse.confirmed_state());
        assert_eq!(&StateRef::new_pending(4), multiverse.last_state());

        multiverse.confirm(&1);
        assert_eq!(&StateRef::new_confirmed(1), multiverse.confirmed_state());
        assert_eq!(&StateRef::new_pending(4), multiverse.last_state());

        multiverse.confirm(&4);
        assert_eq!(&StateRef::new_confirmed(1), multiverse.confirmed_state());

        assert_eq!(&StateRef::new_confirmed(4), multiverse.last_state());

        multiverse.confirm(&3);
        multiverse.confirm(&2);
        assert_eq!(&StateRef::new_confirmed(4), multiverse.confirmed_state());

        assert_eq!(&StateRef::new_confirmed(4), multiverse.last_state());
    }
}
