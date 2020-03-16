use std::marker::PhantomData;

use super::{Node, RebalanceArgs, RebalanceResult, SiblingsArg};
use crate::btreeindex::{
    pages::{borrow::Mutable, PageHandle},
    Keys, KeysMut, PageId, Values, ValuesMut,
};
use crate::BTreeStoreError;
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

pub enum LeafDeleteStatus {
    Ok,
    NeedsRebalance,
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
        // this is safe because we are not reading the data and by setting the length to 0 we are not
        // going to
        let mut uninit = unsafe { Self::from_raw(key_buffer_size, data) };
        uninit.set_len(0);
        uninit
    }

    /// read an already initialized slice of bytes as a leaf node
    pub(crate) unsafe fn from_raw(key_buffer_size: usize, data: T) -> LeafNode<'b, K, T> {
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
                    let split_key = self.keys().get(m - 1 as usize).borrow().clone();

                    for (i, (k, v)) in self
                        .keys()
                        .sub(m - 1..self.keys().len())
                        .into_iter()
                        .zip(self.values().sub(m - 1..self.values().len()).into_iter())
                        .enumerate()
                    {
                        match right_node.as_leaf_mut().insert_key_value::<F>(
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
                    let split_key = self.keys().get(m as usize).borrow().clone();

                    let mut position = 0;
                    for (k, v) in self
                        .keys()
                        .sub(m..pos)
                        .into_iter()
                        .zip(self.values().sub(m..pos).into_iter())
                    {
                        right_node.as_leaf_mut().insert_key_value::<F>(
                            position,
                            k.borrow().clone(),
                            v,
                            None,
                        );
                        position += 1;
                    }

                    right_node.as_leaf_mut().insert_key_value::<F>(
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
                        right_node.as_leaf_mut().insert_key_value::<F>(
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

                    right_node
                        .as_leaf_mut()
                        .insert_key_value::<F>(0, key.clone(), value, None);

                    let mut position = 1;

                    for (k, v) in self
                        .keys()
                        .sub(m..self.keys().len())
                        .into_iter()
                        .zip(self.values().sub(m..self.values().len()).into_iter())
                    {
                        right_node.as_leaf_mut().insert_key_value::<F>(
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

    pub fn rebalance<N: super::NodePageRef>(
        &'b mut self,
        mut args: SiblingsArg<N>,
    ) -> Result<RebalanceResult, BTreeStoreError> {
        let current_len = self.keys().len();

        let result = {
            let left_sibling_handle = match &args {
                SiblingsArg::Left(handle) | SiblingsArg::Both(handle, _) => Some(handle),
                _ => None,
            };

            let right_sibling_handle = match &args {
                SiblingsArg::Right(handle) | SiblingsArg::Both(_, handle) => Some(handle),
                _ => None,
            };

            if current_len < self.lower_bound() {
                // underflow
                if left_sibling_handle
                    .filter(|handle| {
                        handle.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| -> bool {
                            node.as_leaf().has_extra()
                        })
                    })
                    .is_some()
                {
                    RebalanceResult::TookKeyFromLeft
                } else if right_sibling_handle
                    .clone()
                    .filter(|handle| {
                        handle.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| {
                            node.as_leaf().has_extra()
                        })
                    })
                    .is_some()
                {
                    RebalanceResult::TookKeyFromRight
                } else if left_sibling_handle.is_some() {
                    RebalanceResult::MergeIntoLeft
                } else if right_sibling_handle.is_some() {
                    RebalanceResult::MergeIntoSelf
                } else {
                    unreachable!();
                }
            } else {
                // TODO: add error? vs don't do anything
                panic!("node doesn't need rebalance")
            }
        };

        Ok(result)
    }

    pub fn take_key_from_left<'siblings>(
        &mut self,
        mut parent: PageHandle<'siblings, Mutable<'siblings>>,
        anchor: Option<usize>,
        mut sibling: PageHandle<'siblings, Mutable<'siblings>>,
    ) {
        // steal a key from the left sibling
        let current_len = self.keys().len();

        let (stolen_key, stolen_value) =
            sibling.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| {
                let node = node.as_leaf();
                let keys = node.keys();
                let last = keys.len().checked_sub(1).unwrap();
                let stolen_key = keys.get(last);
                let stolen_value = node.values().get(last);
                (stolen_key.borrow().clone(), stolen_value.borrow().clone())
            });

        self.keys_mut()
            .insert(0, &stolen_key)
            .expect("Couldn't insert key at pos 0");
        self.values_mut()
            .insert(0, &stolen_value)
            .expect("Couldn't insert value at pos 0");
        self.set_len(current_len + 1);

        sibling.as_node_mut(self.key_buffer_size, |mut node: Node<K, &mut [u8]>| {
            let mut sibling = node.as_leaf_mut();
            let last = sibling.keys().len().checked_sub(1).unwrap();
            sibling.keys_mut().delete(last).unwrap();
            sibling.values_mut().delete(last).unwrap();
            sibling.set_len(last);
        });

        let pos_to_update_in_parent = anchor.unwrap();

        parent.as_node_mut(self.key_buffer_size, |mut node: Node<K, &mut [u8]>| {
            node.as_internal_mut()
                .update_key(pos_to_update_in_parent, self.keys().get(0).borrow().clone())
                .expect("update key failed: tried to update a key not in range");
        });
    }

    pub fn take_key_from_right<'siblings>(
        &mut self,
        mut parent: PageHandle<'siblings, Mutable<'siblings>>,
        anchor: Option<usize>,
        mut sibling: PageHandle<'siblings, Mutable<'siblings>>,
    ) {
        // steal a key from the right sibling
        let current_len = self.keys().len();

        let (stolen_key, stolen_value) =
            sibling.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| {
                let node = node.as_leaf();
                let keys = node.keys();
                let stolen_key = keys.get(0);
                let stolen_value = node.values().get(0);
                (stolen_key.borrow().clone(), stolen_value.clone())
            });

        // in leaf, keys.len() == values.len()
        let insert_pos = self.keys().len();
        self.keys_mut()
            .append(&stolen_key)
            .expect("Couldn't insert at the end");

        self.values_mut()
            .insert(insert_pos, &stolen_value)
            .expect("Couldn't insert at the end");

        self.set_len(current_len + 1);

        sibling.as_node_mut(self.key_buffer_size, |mut node: Node<K, &mut [u8]>| {
            let mut sibling = node.as_leaf_mut();
            let current_len = sibling.keys().len();
            sibling.keys_mut().delete(0).unwrap();
            sibling.values_mut().delete(0).unwrap();
            sibling.set_len(current_len.checked_sub(1).unwrap());
        });

        let pos_to_update_in_parent = anchor.map_or(0, |anchor| anchor + 1);

        parent.as_node_mut(self.key_buffer_size, |mut node: Node<K, &mut [u8]>| {
            node.as_internal_mut()
                .update_key(
                    pos_to_update_in_parent,
                    sibling.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| {
                        node.as_leaf().keys().get(0).borrow().clone()
                    }),
                )
                .expect("Couldn't update parent key");
        });
    }

    pub fn merge_into_left<'siblings>(
        &mut self,
        parent: PageHandle<'siblings, Mutable<'siblings>>,
        anchor: Option<usize>,
        mut sibling: PageHandle<'siblings, Mutable<'siblings>>,
    ) {
        //merge this into left
        sibling.as_node_mut(self.key_buffer_size, |mut node| {
            let mut merge_target = node.as_leaf_mut();
            for (k, v) in self.keys().into_iter().zip(self.values().into_iter()) {
                // TODO: Create an Append?
                let insert_pos = merge_target.keys().len();
                merge_target
                    .keys_mut()
                    .insert(insert_pos, &k.borrow().clone())
                    .expect("Couldn't insert at the end");
                merge_target
                    .values_mut()
                    .insert(insert_pos, &v.borrow().clone())
                    .expect("Couldn't insert at the end");
                merge_target.set_len(insert_pos + 1);
            }
        });
    }

    pub fn merge_into_self<'siblings>(
        &mut self,
        parent: PageHandle<'siblings, Mutable<'siblings>>,
        anchor: Option<usize>,
        sibling: PageHandle<'siblings, Mutable<'siblings>>,
    ) {
        //merge right into this

        sibling.as_node(self.key_buffer_size, |node: Node<K, &[u8]>| {
            for (k, v) in node
                .as_leaf()
                .keys()
                .into_iter()
                .zip(node.as_leaf().values().into_iter())
            {
                let insert_pos = self.keys().len();
                self.keys_mut()
                    .insert(insert_pos, &k.borrow().clone())
                    .expect("Couldn't insert at the end");
                self.values_mut()
                    .insert(insert_pos, &v.borrow().clone())
                    .expect("Couldn't insert at the end");
                self.set_len(insert_pos + 1);
            }
        });
    }

    pub fn delete<'siblings: 'b>(
        &'b mut self,
        key: &'siblings K,
    ) -> Result<LeafDeleteStatus, BTreeStoreError> {
        match self.keys().binary_search(key) {
            Ok(pos) => {
                self.delete_key_value(pos)
                    .expect("internal error: keys search returned invalid position");
                let current_len = self.keys().len();
                if current_len < self.lower_bound() {
                    Ok(LeafDeleteStatus::NeedsRebalance)
                } else {
                    Ok(LeafDeleteStatus::Ok)
                }
            }
            Err(_) => return Err(BTreeStoreError::KeyNotFound),
        }
    }

    fn delete_key_value(&mut self, pos: usize) -> Result<(), ()> {
        let current_len = self.keys().len();
        self.keys_mut().delete(pos)?;
        self.values_mut().delete(pos)?;

        self.set_len(current_len - 1);
        Ok(())
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

    /// can give one key-value to a neighbour without imbalancing itself
    fn has_extra(&self) -> bool {
        self.values().len() > self.lower_bound()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btreeindex::node::tests::{internal_page, internal_page_mut, pages};
    use crate::btreeindex::pages::borrow::{Immutable, Mutable};
    use crate::btreeindex::pages::Pages;
    use crate::btreeindex::*;
    use crate::tests::U64Key;
    use std::mem::size_of;

    use std::fmt::Debug;

    impl<'a, K: Key, T> Debug for LeafNode<'a, K, T>
    where
        T: AsRef<[u8]>,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "LeafNode {{ max_keys: {}, keys: {} }}",
                self.max_keys,
                self.keys().len()
            )
        }
    }

    impl<'a, T: 'a> PartialEq for LeafNode<'a, U64Key, T>
    where
        T: AsRef<[u8]>,
    {
        fn eq(&self, other: &Self) -> bool {
            let same_keys = self.keys().into_iter().collect::<Vec<U64Key>>()
                == other.keys().into_iter().collect::<Vec<U64Key>>();

            let same_values = self.values().into_iter().collect::<Vec<u64>>()
                == other.values().into_iter().collect::<Vec<u64>>();

            same_keys && same_values
        }
    }

    impl<T> Eq for LeafNode<'_, U64Key, T> where T: AsRef<[u8]> {}

    fn allocate() -> Node<U64Key, MemPage> {
        let page_size = 8 + 8 + size_of::<PageId>() + 3 * size_of::<U64Key>() + 4 * size_of::<V>();
        let page = MemPage::new(page_size);
        Node::new_leaf(std::mem::size_of::<U64Key>(), page)
    }

    fn new_page_mut(
        pages: &Pages,
        page_id: PageId,
        keys: Vec<U64Key>,
        values: Vec<u64>,
    ) -> PageHandle<Mutable> {
        assert_eq!(keys.len(), values.len());

        const NUMBER_OF_KEYS: usize = 3;
        let page_size = crate::btreeindex::node::TAG_SIZE
            + LEN_SIZE
            + NUMBER_OF_KEYS * size_of::<U64Key>()
            + NUMBER_OF_KEYS * size_of::<V>();

        let mut page = pages.mut_page(page_id).unwrap();

        page.as_slice(|slice| {
            Node::<U64Key, &mut [u8]>::new_leaf(size_of::<U64Key>(), slice);
        });

        page.as_node_mut(size_of::<U64Key>(), |mut node| {
            for (k, c) in keys.iter().zip(values.iter()) {
                match node.as_leaf_mut().insert((*k).clone(), *c, &mut allocate) {
                    LeafInsertStatus::Ok => (),
                    _ => panic!("insertion shouldn't split"),
                };
            }
        });

        page
    }

    fn new_page(
        pages: &Pages,
        page_id: PageId,
        keys: Vec<U64Key>,
        values: Vec<u64>,
    ) -> PageHandle<Immutable> {
        {
            new_page_mut(pages, page_id, keys, values);
        }
        pages.get_page(page_id).unwrap()
    }

    #[test]
    fn delete_without_underflow() {
        let pages = pages();
        let mut node = new_page_mut(
            &pages,
            1,
            vec![U64Key(1), U64Key(2), U64Key(3)],
            vec![1, 2, 3],
        );
        node.as_node_mut(size_of::<U64Key>(), |mut node| {
            match node.as_leaf_mut().delete(&U64Key(1)).unwrap() {
                LeafDeleteStatus::Ok => (),
                _ => panic!(),
            }
        });
    }

    #[test]
    fn delete_with_take_from_left() {
        let storage = pages();
        let parent = internal_page_mut(&storage, 3, vec![U64Key(4), U64Key(8)], vec![2, 1, 3]);
        let mut node = new_page_mut(&storage, 1, vec![U64Key(5), U64Key(6)], vec![5, 6]);
        let left_sibling = new_page(
            &storage,
            2,
            vec![U64Key(1), U64Key(2), U64Key(3)],
            vec![1, 2, 3],
        );

        node.as_node_mut(size_of::<U64Key>(), |mut node| {
            match node.as_leaf_mut().delete(&U64Key(5)).unwrap() {
                LeafDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            }
        });

        node.as_node_mut(
            size_of::<U64Key>(),
            |mut node: Node<U64Key, &mut [u8]>| match node
                .as_leaf_mut()
                .rebalance(RebalanceArgs {
                    parent,
                    parent_anchor: Some(0),
                    siblings: SiblingsArg::Left((left_sibling, || {
                        storage.make_shadow(2, 12).unwrap();
                        storage.mut_page(12).unwrap()
                    })),
                })
                .unwrap()
            {
                RebalanceResult::TookKeyFromLeft => (),
                _ => panic!("need took from left"),
            },
        );

        let aux_storage = pages();
        let node_expected = new_page(&aux_storage, 1, vec![U64Key(3), U64Key(6)], vec![3, 6]);
        node.as_node(size_of::<U64Key>(), |before| {
            node_expected.as_node(size_of::<U64Key>(), |node_expected| {
                assert_eq!(before.as_leaf(), node_expected.as_leaf())
            })
        });

        let parent_expected =
            internal_page(&aux_storage, 3, vec![U64Key(3), U64Key(8)], vec![2, 1, 3]);

        storage
            .get_page(3)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                parent_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_internal(), node_expected.as_internal())
                })
            });

        let left_sibling_expected =
            new_page(&aux_storage, 12, vec![U64Key(1), U64Key(2)], vec![1, 2]);

        storage
            .get_page(12)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                left_sibling_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_leaf(), node_expected.as_leaf())
                })
            });
    }

    #[test]
    fn delete_with_take_from_right() {
        let storage = pages();
        let parent = internal_page_mut(&storage, 3, vec![U64Key(3), U64Key(8)], vec![1, 2, 3]);
        let mut node = new_page_mut(&storage, 1, vec![U64Key(1), U64Key(2)], vec![1, 2]);
        let right_sibling = new_page(
            &storage,
            2,
            vec![U64Key(4), U64Key(5), U64Key(6)],
            vec![4, 5, 6],
        );

        node.as_node_mut(size_of::<U64Key>(), |mut node| {
            match node.as_leaf_mut().delete(&U64Key(1)).unwrap() {
                LeafDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            }
        });

        node.as_node_mut(
            size_of::<U64Key>(),
            |mut node: Node<U64Key, &mut [u8]>| match node
                .as_leaf_mut()
                .rebalance(RebalanceArgs {
                    parent,
                    parent_anchor: None,
                    siblings: SiblingsArg::Right((right_sibling, || {
                        storage.make_shadow(2, 12);
                        storage.mut_page(12).unwrap()
                    })),
                })
                .unwrap()
            {
                RebalanceResult::TookKeyFromRight => (),
                _ => panic!("need took from right"),
            },
        );

        let aux_storage = pages();
        let node_expected = new_page(&aux_storage, 1, vec![U64Key(2), U64Key(4)], vec![2, 4]);
        node.as_node(size_of::<U64Key>(), |before| {
            node_expected.as_node(size_of::<U64Key>(), |node_expected| {
                assert_eq!(before.as_leaf(), node_expected.as_leaf())
            })
        });

        let parent_expected =
            internal_page(&aux_storage, 3, vec![U64Key(5), U64Key(8)], vec![1, 2, 3]);
        storage
            .get_page(3)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                parent_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_internal(), node_expected.as_internal())
                })
            });

        let right_sibling_expected =
            new_page(&aux_storage, 2, vec![U64Key(5), U64Key(6)], vec![5, 6]);

        storage
            .get_page(12)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                right_sibling_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_leaf(), node_expected.as_leaf())
                })
            });
    }

    #[test]
    fn delete_with_left_merge() {
        let storage = pages();
        let parent = internal_page_mut(&storage, 3, vec![U64Key(3), U64Key(8)], vec![2, 1, 3]);
        let mut node = new_page_mut(&storage, 1, vec![U64Key(4), U64Key(5)], vec![4, 5]);
        let left_sibling = new_page(&storage, 2, vec![U64Key(1), U64Key(2)], vec![1, 2]);

        node.as_node_mut(size_of::<U64Key>(), |mut node| {
            match node.as_leaf_mut().delete(&U64Key(4)).unwrap() {
                LeafDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            }
        });

        node.as_node_mut(
            size_of::<U64Key>(),
            |mut node: Node<U64Key, &mut [u8]>| match node
                .as_leaf_mut()
                .rebalance(RebalanceArgs {
                    parent,
                    parent_anchor: Some(0),
                    siblings: SiblingsArg::Left((left_sibling, || {
                        storage.make_shadow(2, 12).unwrap();
                        storage.mut_page(12).unwrap()
                    })),
                })
                .unwrap()
            {
                RebalanceResult::MergeIntoLeft => (),
                _ => panic!("need merge into left"),
            },
        );

        let aux_storage = pages();
        let left_sibling_expected = new_page(
            &aux_storage,
            2,
            vec![U64Key(1), U64Key(2), U64Key(5)],
            vec![1, 2, 5],
        );

        storage
            .get_page(12)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                left_sibling_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_leaf(), node_expected.as_leaf())
                })
            });

        let parent_expected =
            internal_page(&aux_storage, 3, vec![U64Key(3), U64Key(8)], vec![2, 1, 3]);

        storage
            .get_page(3)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                parent_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_internal(), node_expected.as_internal())
                })
            });
    }

    #[test]
    fn delete_with_right_merge() {
        let storage = pages();

        let parent = internal_page_mut(&storage, 3, vec![U64Key(3), U64Key(8)], vec![1, 2, 3]);
        let mut node = new_page_mut(&storage, 1, vec![U64Key(1), U64Key(2)], vec![1, 2]);
        let right_sibling = new_page(&storage, 2, vec![U64Key(4), U64Key(5)], vec![4, 5]);

        node.as_node_mut(
            size_of::<U64Key>(),
            |mut node: Node<U64Key, &mut [u8]>| match node.as_leaf_mut().delete(&U64Key(2)).unwrap()
            {
                LeafDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            },
        );

        node.as_node_mut(
            size_of::<U64Key>(),
            |mut node: Node<U64Key, &mut [u8]>| match node
                .as_leaf_mut()
                .rebalance(RebalanceArgs {
                    parent,
                    parent_anchor: None,
                    siblings: SiblingsArg::Right((right_sibling, || {
                        storage.make_shadow(2, 12).unwrap();
                        storage.mut_page(12).unwrap()
                    })),
                })
                .unwrap()
            {
                RebalanceResult::MergeIntoSelf => (),
                _ => panic!("need merge into self"),
            },
        );

        let aux_storage = pages();
        let node_expected = new_page(
            &aux_storage,
            2,
            vec![U64Key(1), U64Key(4), U64Key(5)],
            vec![1, 4, 5],
        );
        node.as_node(size_of::<U64Key>(), |before| {
            node_expected.as_node(size_of::<U64Key>(), |node_expected| {
                assert_eq!(before.as_leaf(), node_expected.as_leaf())
            })
        });

        let parent_expected =
            internal_page(&aux_storage, 3, vec![U64Key(3), U64Key(8)], vec![1, 2, 3]);
        storage
            .get_page(3)
            .unwrap()
            .as_node(size_of::<U64Key>(), |before| {
                parent_expected.as_node(size_of::<U64Key>(), |node_expected| {
                    assert_eq!(before.as_internal(), node_expected.as_internal())
                })
            });
    }
}
