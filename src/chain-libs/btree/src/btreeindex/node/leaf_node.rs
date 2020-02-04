use std::marker::PhantomData;

use super::Node;
use crate::btreeindex::{Keys, KeysMut, PageId, Values, ValuesMut};
use crate::Key;
use crate::MemPage;
use crate::Value as V;
use byteorder::ByteOrder as _;
use byteorder::LittleEndian;
use std::borrow::Borrow;

use std::convert::{TryFrom, TryInto};
use std::mem::size_of;

pub(crate) enum LeafInsertStatus<K> {
    Ok,
    Split(K, Node<K, MemPage>),
    DuplicatedKey(K),
}

/// LeafNode is a wrapper over a slice of bytes (T). The layout is the following
/// LEN | KEYS | VALUES(u64)
/// For the time being, is assumed that the memory region is aligned to an 8 byte boundary,
/// and that each key (key_buffer_size) is a multiple of 8, although it would probably work anyway?

pub struct LeafNode<'a, K, T: 'a> {
    max_keys: usize,
    key_buffer_size: usize,
    data: T,
    phantom: PhantomData<&'a [K]>,
}

const LEN_START: usize = 0;
const LEN_SIZE: usize = 8;
const KEYS_START: usize = LEN_START + LEN_SIZE;

impl<'b, K, T> LeafNode<'b, K, T>
where
    K: Key,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    /// mutate the slice of bytes so it is a valid leaf node
    pub(crate) fn init(key_buffer_size: usize, data: T) -> LeafNode<'b, K, T> {
        let mut uninit = Self::from_raw(key_buffer_size, data);
        uninit.set_len(0);
        uninit
    }

    /// read an already initialized slice of bytes as a leaf node
    pub(crate) fn from_raw(key_buffer_size: usize, data: T) -> LeafNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);
        assert!(key_buffer_size % 8 == 0);

        let size_per_key = key_buffer_size + size_of::<V>();
        let extra_size = LEN_SIZE;

        let max_keys = (usize::try_from(data.as_ref().len()).unwrap()
            - usize::try_from(extra_size).unwrap())
            / size_per_key;

        LeafNode {
            max_keys,
            key_buffer_size,
            data,
            phantom: PhantomData,
        }
    }

    /// insert given key and value, the allocate function is used in case a split is necessary
    pub(crate) fn insert(
        &mut self,
        key: K,
        value: V,
        allocate: impl FnMut() -> Node<K, MemPage>,
    ) -> LeafInsertStatus<K> {
        match self.keys().binary_search(&key) {
            Ok(_) => LeafInsertStatus::DuplicatedKey(key),
            Err(index) => {
                self.insert_key_value(index.try_into().unwrap(), key, value, Some(allocate))
            }
        }
    }

    fn insert_key_value<F>(
        &mut self,
        pos: usize,
        key: K,
        value: V,
        allocate: Option<F>,
    ) -> LeafInsertStatus<K>
    where
        F: FnMut() -> Node<K, MemPage>,
    {
        let current_len = self.keys().len();
        let m = self.lower_bound();

        let result = { self.keys_mut().insert(pos, &key) };
        match result {
            Ok(()) => {
                self.values_mut().insert(pos, &value).unwrap();
                self.set_len(current_len.checked_add(1).unwrap());
                LeafInsertStatus::Ok
            }
            Err(()) => {
                let mut right_node = allocate.unwrap()();

                if pos < m.try_into().unwrap() {
                    let split_key = self.keys().get(m - 1 as usize).unwrap().borrow().clone();

                    for (i, (k, v)) in self
                        .keys()
                        .sub(m - 1..self.keys().len())
                        .into_iter()
                        .zip(self.values().sub(m - 1..self.values().len()).into_iter())
                        .enumerate()
                    {
                        match right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                            i,
                            k.borrow().clone(),
                            v,
                            None,
                        ) {
                            LeafInsertStatus::Ok => (),
                            _ => unreachable!(),
                        }
                    }

                    self.set_len(m.saturating_sub(1) as usize);

                    match self.insert_key_value::<F>(pos, key, value, None) {
                        LeafInsertStatus::Ok => (),
                        _ => unreachable!(),
                    };

                    LeafInsertStatus::Split(split_key.clone(), right_node)
                } else if pos > m.try_into().unwrap() {
                    let split_key = self.keys().get(m as usize).unwrap().borrow().clone();

                    let mut position = 0;
                    for (k, v) in self
                        .keys()
                        .sub(m..pos)
                        .into_iter()
                        .zip(self.values().sub(m..pos).into_iter())
                    {
                        right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                            position,
                            k.borrow().clone(),
                            v,
                            None,
                        );
                        position += 1;
                    }

                    right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                        position,
                        key.clone(),
                        value.clone(),
                        None,
                    );
                    position += 1;

                    for (k, v) in self
                        .keys()
                        .sub(pos..self.keys().len())
                        .into_iter()
                        .zip(self.values().sub(pos..self.values().len()).into_iter())
                    {
                        right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                            position,
                            k.borrow().clone(),
                            v,
                            None,
                        );
                        position += 1;
                    }

                    self.set_len(m as usize);

                    LeafInsertStatus::Split(split_key.clone(), right_node)
                } else {
                    // pos == m

                    let split_key = key.clone();

                    right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                        0,
                        key.clone(),
                        value,
                        None,
                    );

                    let mut position = 1;

                    for (k, v) in self
                        .keys()
                        .sub(m..self.keys().len())
                        .into_iter()
                        .zip(self.values().sub(m..self.values().len()).into_iter())
                    {
                        right_node.as_leaf_mut().unwrap().insert_key_value::<F>(
                            position,
                            k.borrow().clone(),
                            v,
                            None,
                        );

                        position += 1;
                    }

                    // Truncate left(self) node to have `m` elements
                    self.set_len(m as usize);

                    LeafInsertStatus::Split(split_key, right_node)
                }
            }
        }
    }

    fn values_mut(&mut self) -> ValuesMut {
        let len = self.keys().len();

        let base = KEYS_START + (self.max_keys * self.key_buffer_size);
        let data = &mut self.data.as_mut()[base..base + self.max_keys * size_of::<V>()];

        ValuesMut::new_static_size(data, len)
    }

    fn keys_mut(&mut self) -> KeysMut<K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);
        let data =
            &mut self.data.as_mut()[KEYS_START..KEYS_START + self.max_keys * self.key_buffer_size];

        KeysMut::new_dynamic_size(data, len.try_into().unwrap(), self.key_buffer_size)
    }

    fn set_len(&mut self, new_len: usize) {
        let new_len = u64::try_from(new_len).unwrap();
        self.data.as_mut()[0..LEN_SIZE].copy_from_slice(&new_len.to_le_bytes());
    }
}

impl<'b, K, T> LeafNode<'b, K, T>
where
    K: Key,
    T: AsRef<[u8]> + 'b,
{
    /// same as from_raw but for inmutable slices
    pub(crate) fn view(key_buffer_size: usize, data: T) -> LeafNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);

        let size_per_key = key_buffer_size + size_of::<V>();
        let extra_size = LEN_SIZE;

        let max_keys = (usize::try_from(data.as_ref().len()).unwrap()
            - usize::try_from(extra_size).unwrap())
            / size_per_key;

        LeafNode {
            max_keys,
            key_buffer_size,
            data,
            phantom: PhantomData,
        }
    }

    /// minimum number of keys a leaf node can have
    fn lower_bound(&self) -> usize {
        let upper_bound = self.max_keys;
        let div = upper_bound / 2;
        if upper_bound % 2 == 1 {
            div + 1
        } else {
            div
        }
    }

    /// inmutable view over the keys
    pub(crate) fn keys(&self) -> Keys<K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);
        let data =
            &self.data.as_ref()[KEYS_START..KEYS_START + self.max_keys * self.key_buffer_size];

        Keys::new_dynamic_size(data, len.try_into().unwrap(), self.key_buffer_size)
    }

    /// inmutable view over the values
    pub(crate) fn values(&self) -> Values {
        let len = self.keys().len();

        let base = KEYS_START + (self.max_keys * self.key_buffer_size);
        let data: &[u8] = &self.data.as_ref()[base..base + self.max_keys * size_of::<V>()];

        Values::new_static_size(data, len)
    }
}
