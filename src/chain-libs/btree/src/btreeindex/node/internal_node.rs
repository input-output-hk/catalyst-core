use super::{Node, NodeRef, NodeRefMut, RebalanceResult, RebalanceSiblingArg, SiblingsArg};
use crate::btreeindex::{Children, ChildrenMut, Keys, KeysMut, PageId};
use crate::{BTreeStoreError, FixedSize, MemPage};
use byteorder::{ByteOrder as _, LittleEndian};
use std::borrow::Borrow;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
use std::mem::size_of;

const LEN_START: usize = 0;
const LEN_SIZE: usize = 8;
// const CHILDREN_PADDING: usize = 0;

pub(crate) struct InternalNode<'a, K, T: 'a> {
    max_keys: usize,
    data: T,
    phantom: PhantomData<&'a [K]>,
}
/// InternalNode is a wrapper over a slice of bytes (T). The layout is the following
/// LEN | KEYS | CHILDREN(u64)
/// For the time being, is assumed that the memory region is aligned to an 8 byte boundary,
/// and that each key (key_buffer_size) is a multiple of 8, although it would probably work anyway?
/// both KEYS and CHILDREN are themselves arrays. KEYS[0] has CHILDREN[0] as left child and CHILDREN[1] as
/// right child.
/// Also LEN == LEN(KEYS) == LEN(CHILDREN) - 1
pub(crate) enum InternalInsertStatus<K> {
    Ok,
    Split(K, Node<K, MemPage>),
    DuplicatedKey(K),
}

#[derive(Debug)]
pub(crate) enum InternalDeleteStatus {
    Ok,
    NeedsRebalance,
}

impl<'b, K, T> InternalNode<'b, K, T>
where
    K: FixedSize,
    T: AsRef<[u8]> + AsMut<[u8]> + 'b,
{
    /// Init the given slice (mutating it) so it is a valid (empty) InternalNode that
    /// can be later read with `from_raw`
    pub fn init(buffer: T) -> InternalNode<'b, K, T> {
        // this is safe because we are not reading the data and by setting the length to 0 we are not
        // going to
        let mut uninit = unsafe { Self::from_raw(buffer) };
        uninit.set_len(0);
        uninit
    }

    /// mutable version of node interpretated over the given slice
    /// this shouldn't be called before calling `init`
    // TODO: add more rigorous type checking?
    pub unsafe fn from_raw(data: T) -> InternalNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);
        assert!(!data.as_ref().is_empty());

        let size_per_key = K::max_size() + size_of::<PageId>();
        let extra_size = LEN_SIZE - LEN_START;

        let max_keys = (data.as_ref().len() - extra_size - size_of::<PageId>()) / size_per_key;

        InternalNode {
            max_keys,
            data,
            phantom: PhantomData,
        }
    }

    // The first insertion of an InternalNode is different, because in general we can insert
    // only new keys and the right child. When the node has only one key, we need two children so
    // the first insertion must insert two keys
    pub fn insert_first(&mut self, key: K, left: PageId, right: PageId) {
        assert_eq!(self.keys().len(), 0);
        self.keys_mut()
            .append(&key)
            .expect("couldn't insert first key");
        let mut children_mut = self.children_mut();

        children_mut
            .append(&left)
            .expect("couldn't insert first child");

        children_mut
            .append(&right)
            .expect("couldn't insert second child");

        self.set_len(1);
    }

    /// function to call after inserted the first key with `insert_first`. It will insert key with node_id as right child
    pub fn insert(
        &mut self,
        key: K,
        node_id: PageId,
        allocate: impl FnMut() -> Node<K, MemPage>,
    ) -> InternalInsertStatus<K> {
        // Non empty, maybe encapsulate in some kind of state machine
        assert!(self.keys().len() > 0);
        match self.keys().binary_search(&key) {
            Ok(_) => InternalInsertStatus::DuplicatedKey(key),
            Err(index) => self.insert_key_child(index, key, node_id, allocate),
        }
    }

    pub fn children_mut(&mut self) -> ChildrenMut<'_> {
        let len = if self.keys().len() > 0 {
            self.keys().len().checked_add(1).unwrap() as usize
        } else {
            0
        };

        let base = LEN_SIZE + (self.max_keys * K::max_size());
        let data = &mut self.data.as_mut()
            [base..base + (self.max_keys.checked_add(1).unwrap()) * size_of::<PageId>()];

        ChildrenMut::new_static_size(data, len)
    }

    fn keys_mut(&mut self) -> KeysMut<'_, K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);

        let data = &mut self.data.as_mut()[LEN_SIZE..LEN_SIZE + self.max_keys * K::max_size()];

        KeysMut::new_dynamic_size(data, len.try_into().unwrap(), K::max_size())
    }

    fn insert_key_child(
        &mut self,
        pos: usize,
        key: K,
        node_id: PageId,
        mut allocate: impl FnMut() -> Node<K, MemPage>,
    ) -> InternalInsertStatus<K> {
        let current_len = self.keys().len();
        let m = self.lower_bound().checked_sub(1).unwrap();

        let result = { self.keys_mut().insert(pos, &key) };
        match result {
            Ok(()) => {
                self.children_mut()
                    .insert(pos.checked_add(1).unwrap(), &node_id)
                    .expect("key inserted but child insertion failed");
                self.set_len(current_len.checked_add(1).unwrap());
                InternalInsertStatus::Ok
            }
            Err(()) => {
                let mut right_node = allocate();

                // The following 3 branches do essentially the same
                // we need to keep m keys on the current node, and send the rest to the right node
                // the idea of the algorithm would be to insert the new key at pos by moving everything after
                // to the right node and then move from self.keys()[m..] to the right
                // as there is no extra space between the keys and the children, we need to move directly
                // the extra keys before actually doing the insert

                // the key would be inserted in the first half
                use std::cmp::Ordering;
                match pos.cmp(&m) {
                    Ordering::Less => {
                        let mut right_node_internal = right_node.as_internal_mut();
                        let split_key = self.keys().get(m - 1 as usize).borrow().clone();

                        let mut keys_mut = right_node_internal.keys_mut();
                        for k in self.keys().sub(m..self.keys().len()).iter() {
                            keys_mut.append(k.borrow()).unwrap();
                        }

                        let new_len = keys_mut.len();

                        let mut children_mut = right_node_internal.children_mut();

                        for c in self
                            .children()
                            .sub(m as usize..self.children().len())
                            .iter()
                        {
                            children_mut
                                .append(c.borrow())
                                .expect("Couldn't insert children");
                        }

                        right_node_internal.set_len(new_len);

                        self.set_len(m);
                        self.keys_mut().insert(pos, &key).unwrap();
                        self.children_mut()
                            .insert(pos.checked_add(1).unwrap(), &node_id)
                            .unwrap();

                        InternalInsertStatus::Split(split_key, right_node)
                    }
                    Ordering::Greater => {
                        let mut right_internal_node = right_node.as_internal_mut();
                        let split_key = self.keys().get(m as usize).borrow().clone();

                        let mut keys_mut = right_internal_node.keys_mut();
                        for k in self.keys().sub(m + 1..pos as usize).iter() {
                            keys_mut.append(k.borrow()).unwrap();
                        }

                        keys_mut.append(&key).unwrap();

                        for k in self.keys().sub(pos as usize..self.keys().len()).iter() {
                            keys_mut.append(k.borrow()).unwrap();
                        }

                        let new_len = keys_mut.len();
                        let mut children_mut = right_internal_node.children_mut();

                        for c in self
                            .children()
                            .sub(m as usize + 1..pos as usize + 1)
                            .iter()
                            .chain(Some(node_id).into_iter())
                            .chain(
                                self.children()
                                    .sub(pos as usize + 1..self.children().len())
                                    .iter(),
                            )
                        {
                            children_mut.append(c.borrow()).unwrap();
                        }

                        right_internal_node.set_len(new_len);

                        // Truncate node to have m elements
                        self.set_len(m as usize);
                        InternalInsertStatus::Split(split_key, right_node)
                    }
                    Ordering::Equal => {
                        let mut right_internal_node = right_node.as_internal_mut();

                        let split_key = key;

                        let mut keys_mut = right_internal_node.keys_mut();
                        for k in self.keys().sub(m as usize..self.keys().len()).iter() {
                            keys_mut.append(k.borrow()).unwrap();
                        }

                        let new_len = keys_mut.len();
                        let mut children_mut = right_internal_node.children_mut();

                        for c in Some(node_id).into_iter().chain(
                            self.children()
                                .sub(m as usize + 1..self.children().len())
                                .iter(),
                        ) {
                            children_mut.append(c.borrow()).unwrap();
                        }

                        right_internal_node.set_len(new_len);
                        self.set_len(m as usize);

                        InternalInsertStatus::Split(split_key, right_node)
                    }
                }
            }
        }
    }

    pub fn delete_key_children(&mut self, pos: usize) -> InternalDeleteStatus {
        let current_len = self.keys().len();
        self.keys_mut()
            .delete(pos)
            .expect("Couldn't delete last key");
        self.children_mut()
            .delete(pos + 1)
            .expect("Couldn't delete last child");
        self.set_len(current_len - 1);

        if self.children().len() < self.lower_bound() {
            InternalDeleteStatus::NeedsRebalance
        } else {
            InternalDeleteStatus::Ok
        }
    }

    pub fn rebalance<N: super::NodeRef>(
        self,
        args: SiblingsArg<N>,
    ) -> Result<RebalanceResult<Self>, BTreeStoreError> {
        let result = {
            let left_sibling_handle = match &args {
                SiblingsArg::Left(handle) | SiblingsArg::Both(handle, _) => Some(handle),
                _ => None,
            };

            let right_sibling_handle = match &args {
                SiblingsArg::Right(handle) | SiblingsArg::Both(_, handle) => Some(handle),
                _ => None,
            };

            let has_extra = |handle: &&N| -> bool {
                handle.as_node(|node: Node<K, &[u8]>| node.as_internal().has_extra())
            };

            if self.children().len() < self.lower_bound() {
                // underflow
                if left_sibling_handle.filter(has_extra).is_some() {
                    RebalanceResult::TakeFromLeft(RebalanceSiblingArg::new(self))
                } else if right_sibling_handle.clone().filter(has_extra).is_some() {
                    RebalanceResult::TakeFromRight(RebalanceSiblingArg::new(self))
                } else if left_sibling_handle.is_some() {
                    RebalanceResult::MergeIntoLeft(RebalanceSiblingArg::new(self))
                } else if right_sibling_handle.is_some() {
                    RebalanceResult::MergeIntoSelf(RebalanceSiblingArg::new(self))
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

    pub fn update_key(&mut self, pos: usize, key: K) -> Result<(), ()> {
        self.keys_mut().update(pos, &key)
    }

    fn set_len(&mut self, new_len: usize) {
        let new_len = u64::try_from(new_len).unwrap();
        LittleEndian::write_u64(&mut self.data.as_mut()[0..LEN_SIZE], new_len);
    }
}

impl<'b, K, T> InternalNode<'b, K, T>
where
    K: FixedSize,
    T: AsRef<[u8]> + 'b,
{
    pub fn view(data: T) -> InternalNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);
        assert!(!data.as_ref().is_empty());

        let size_per_key = K::max_size() + size_of::<PageId>();
        let extra_size = LEN_SIZE - LEN_START;

        let max_keys = (data.as_ref().len() - extra_size - size_of::<PageId>()) / size_per_key;

        InternalNode {
            max_keys,
            data,
            phantom: PhantomData,
        }
    }

    fn lower_bound(&self) -> usize {
        let upper_bound = self.max_keys.checked_add(1).unwrap();
        let div = upper_bound / 2;
        if upper_bound % 2 == 1 {
            div + 1
        } else {
            div
        }
    }

    pub(crate) fn children(&self) -> Children<'_> {
        let len = if self.keys().len() > 0 {
            self.keys().len().checked_add(1).unwrap() as usize
        } else {
            0
        };

        let base = LEN_SIZE + (self.max_keys * K::max_size());
        let data = &self.data.as_ref()
            [base..base + (self.max_keys.checked_add(1).unwrap()) * size_of::<PageId>()];

        Children::new_static_size(data, len)
    }

    pub(crate) fn keys(&self) -> Keys<'_, K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);

        let data = &self.data.as_ref()[LEN_SIZE..LEN_SIZE + self.max_keys * K::max_size()];

        Keys::new_dynamic_size(data, len.try_into().unwrap(), K::max_size())
    }

    fn has_extra(&self) -> bool {
        self.children().len() > self.lower_bound()
    }
}

impl<'b, K, T> RebalanceSiblingArg<super::marker::TakeFromLeft, InternalNode<'b, K, T>>
where
    K: FixedSize,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    pub fn take_key_from_left(
        mut self,
        mut parent: impl NodeRefMut,
        anchor: usize,
        mut sibling: impl NodeRefMut,
    ) -> InternalNode<'b, K, T> {
        // steal a key from the left sibling through parent
        let current_len = self.node.keys().len();

        let (new_anchor_key, new_first_child) = sibling.as_node(|node: Node<K, &[u8]>| {
            let node = node.as_internal();
            let keys = node.keys();
            let last = keys.len().checked_sub(1).unwrap();
            let stolen_key = keys.get(last);
            let stolen_child = node.children().get(last + 1);
            (stolen_key.borrow().clone(), *stolen_child.borrow())
        });

        let new_first_key = parent.as_node(|node: Node<K, &[u8]>| {
            let node = node.as_internal();
            let keys = node.keys();
            let stolen_key = keys.get(anchor);
            stolen_key.borrow().clone()
        });

        self.node
            .keys_mut()
            .insert(0, &new_first_key)
            .expect("Couldn't insert key at pos 0");
        self.node
            .children_mut()
            .insert(0, &new_first_child)
            .expect("Couldn't insert child at pos 0");
        self.node.set_len(current_len + 1);

        parent.as_node_mut(|mut node| {
            node.as_internal_mut()
                .update_key(anchor, new_anchor_key.borrow().clone())
                .unwrap();
        });

        sibling.as_node_mut(|mut node: Node<K, &mut [u8]>| {
            let mut sibling = node.as_internal_mut();
            let last = sibling.keys().len().checked_sub(1).unwrap();
            sibling.keys_mut().delete(last).unwrap();
            sibling.children_mut().delete(last + 1).unwrap();
            sibling.set_len(last);
        });
        self.node
    }
}

impl<'b, K, T> RebalanceSiblingArg<super::marker::TakeFromRight, InternalNode<'b, K, T>>
where
    K: FixedSize,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    pub fn take_key_from_right(
        mut self,
        mut parent: impl NodeRefMut,
        anchor: Option<usize>,
        mut sibling: impl NodeRefMut,
    ) -> InternalNode<'b, K, T> {
        // steal a key from the right sibling though parent
        let current_len = self.node.keys().len();

        let (new_right_anchor_key, new_last_child) = sibling.as_node(|node: Node<K, &[u8]>| {
            let node = node.as_internal();
            let keys = node.keys();
            let stolen_key = keys.get(0);
            let stolen_child = node.children().get(0);
            (stolen_key.borrow().clone(), *stolen_child.borrow())
        });

        let right_anchor_pos = anchor.map(|a| a + 1).unwrap_or(0);

        let new_last_key = parent.as_node(|node: Node<K, &[u8]>| {
            node.as_internal()
                .keys()
                .get(right_anchor_pos)
                .borrow()
                .clone()
        });

        parent.as_node_mut(|mut node| {
            node.as_internal_mut()
                .update_key(right_anchor_pos, new_right_anchor_key.borrow().clone())
                .unwrap()
        });

        let insert_pos = self.node.keys().len();
        self.node
            .keys_mut()
            .append(new_last_key.borrow())
            .expect("Couldn't append key");
        self.node
            .children_mut()
            .insert(insert_pos + 1, &new_last_child)
            .expect("Couldn't append child");
        self.node.set_len(current_len + 1);

        sibling.as_node_mut(|mut node: Node<K, &mut [u8]>| {
            let mut sibling = node.as_internal_mut();
            let current_len = sibling.keys().len();
            sibling
                .keys_mut()
                .delete(0)
                .expect("Couldn't delete key at pos 0");
            sibling
                .children_mut()
                .delete(0)
                .expect("Couldn't delete child at pos 0");
            sibling.set_len(current_len.checked_sub(1).unwrap());
        });

        self.node
    }
}

impl<'b, K, T> RebalanceSiblingArg<super::marker::MergeIntoLeft, InternalNode<'b, K, T>>
where
    K: FixedSize,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    pub fn merge_into_left<'parent: 'b>(
        mut self,
        parent: impl NodeRefMut + 'parent,
        anchor: Option<usize>,
        mut sibling: impl NodeRefMut,
    ) -> Result<InternalNode<'b, K, T>, BTreeStoreError> {
        //merge this into left
        let anchor_key = parent.as_node(|node: Node<K, &[u8]>| {
            node.as_internal()
                .keys()
                .get(anchor.unwrap())
                .borrow()
                .clone()
        });

        let mut anchor_key_buf = vec![0; K::max_size()];
        let borrowed_key = match anchor_key
            .write(&mut anchor_key_buf)
            .and_then(|_| K::read(&anchor_key_buf[..]))
        {
            Ok(key) => key,
            Err(_) => return Err(BTreeStoreError::InconsistentWriteRead),
        };

        let keys = self.node.keys();
        let children = self.node.children();

        sibling.as_node_mut(|mut node| {
            for (k, v) in Some(borrowed_key)
                .into_iter()
                .chain(keys.iter())
                .zip(children.iter())
            {
                let mut merge_target = node.as_internal_mut();
                let insert_pos = merge_target.keys().len();
                merge_target
                    .keys_mut()
                    .append(k.borrow())
                    .expect("Couldn't append key");
                merge_target
                    .children_mut()
                    .insert(insert_pos + 1, &v)
                    .expect("Couldn't append child");
                merge_target.set_len(insert_pos + 1);
            }
        });

        self.node.set_len(0);
        Ok(self.node)
    }
}

impl<'b, K, T> RebalanceSiblingArg<super::marker::MergeIntoSelf, InternalNode<'b, K, T>>
where
    K: FixedSize,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    /// take all the keys from the right sibling and append them to this node, this doesn't mutate the right
    /// sibling because in the full algorithm it will be deleted afterwards anyway
    pub fn merge_into_self(
        mut self,
        parent: impl NodeRefMut,
        anchor: Option<usize>,
        sibling: impl NodeRef,
    ) -> Result<InternalNode<'b, K, T>, BTreeStoreError> {
        //merge right into this
        let anchor_key = parent.as_node(|node: Node<K, &[u8]>| {
            node.as_internal()
                .keys()
                .get(anchor.map_or(0, |a| a + 1))
                .borrow()
                .clone()
        });

        sibling.as_node(|node: Node<K, &[u8]>| {
            let mut anchor_key_buf = vec![0; K::max_size()];
            let borrowed_key = match anchor_key
                .write(&mut anchor_key_buf)
                .and_then(|_| K::read(&anchor_key_buf[..]))
            {
                Ok(key) => key,
                Err(_) => return Err(BTreeStoreError::InconsistentWriteRead),
            };

            for (k, v) in Some(borrowed_key)
                .into_iter()
                .chain(node.as_internal().keys().iter())
                .zip(node.as_internal().children().iter())
            {
                let insert_pos = self.node.keys().len();
                self.node
                    .keys_mut()
                    .append(k.borrow())
                    .expect("Couldn't append key");
                self.node
                    .children_mut()
                    .insert(insert_pos + 1, &v)
                    .expect("Couldn't append child");
                self.node.set_len(insert_pos + 1);
            }

            Ok(())
        })?;

        Ok(self.node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btreeindex::node::tests::{
        allocate_internal as allocate, internal_page, internal_page_mut, pages,
    };
    use crate::tests::U64Key;

    impl<T: AsRef<[u8]> + AsMut<[u8]>> InternalNode<'_, U64Key, T> {
        fn delete(&mut self, key: &U64Key) -> Result<InternalDeleteStatus, BTreeStoreError> {
            match self.keys().binary_search(key) {
                Ok(pos) => Ok(self.delete_key_children(pos)),
                Err(_pos) => Err(BTreeStoreError::KeyNotFound),
            }
        }
    }
    // TEMPORAL
    use std::fmt::Debug;
    impl<'a, K: FixedSize, T> Debug for InternalNode<'a, K, T>
    where
        T: AsRef<[u8]>,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let keys: Vec<K> = self.keys().iter().map(|k| k.borrow().clone()).collect();
            write!(
                f,
                "Internal Node {{ max_keys: {}, keys: {:?} }}",
                self.max_keys, keys
            )
        }
    }

    impl<'a, T: 'a> PartialEq for InternalNode<'a, U64Key, T>
    where
        T: AsRef<[u8]>,
    {
        fn eq(&self, other: &Self) -> bool {
            let same_keys = self.keys().iter().collect::<Vec<U64Key>>()
                == other.keys().iter().collect::<Vec<U64Key>>();

            let same_children = self.children().iter().collect::<Vec<u32>>()
                == other.children().iter().collect::<Vec<u32>>();

            same_keys && same_children
        }
    }

    impl<T> Eq for InternalNode<'_, U64Key, T> where T: AsRef<[u8]> {}

    #[test]
    fn delete_without_underflow() {
        let input = [2, 3];
        let page_size = 8 + 8 + 3 * size_of::<U64Key>() + 4 * size_of::<PageId>();

        let buffer = MemPage::new(page_size);
        let mut node: Node<U64Key, MemPage> = Node::new_internal(buffer);

        node.as_internal_mut().insert_first(U64Key(1), 0, 1);

        for i in input.iter() {
            match node
                .as_internal_mut()
                .insert(U64Key(*i as u64), *i, &mut allocate)
            {
                InternalInsertStatus::Ok => (),
                _ => panic!("insertion shouldn't split"),
            };
        }

        match node.as_internal_mut().delete(&U64Key(1)).unwrap() {
            InternalDeleteStatus::Ok => (),
            _ => panic!(),
        }
    }

    #[test]
    fn delete_with_take_from_left() {
        let before_pages = pages();
        let mut node = internal_page_mut(
            &before_pages,
            22,
            vec![U64Key(15), U64Key(17)],
            vec![1u32, 2, 3],
        );
        let parent = internal_page_mut(
            &before_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![21, 22, 23, 24],
        );
        let left_sibling = internal_page(
            &before_pages,
            21,
            vec![U64Key(1), U64Key(3), U64Key(5)],
            vec![31, 32, 33, 34],
        );

        node.as_node_mut(|mut node| {
            let mut node = node.as_internal_mut();
            assert!(node.keys().len() == 2);
            match node.delete(&U64Key(15)).unwrap() {
                InternalDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            }

            match node.rebalance(SiblingsArg::Left(left_sibling)).unwrap() {
                RebalanceResult::TakeFromLeft(add_params) => {
                    before_pages.make_shadow(21, 31).unwrap();
                    add_params.take_key_from_left(parent, 0, before_pages.mut_page(31).unwrap());
                }
                _ => panic!(),
            }
        });

        let auxiliar_pages = pages();
        let node_expected = internal_page(
            &auxiliar_pages,
            22,
            vec![U64Key(10), U64Key(17)],
            vec![34, 1, 3],
        );

        node.as_node(|before| {
            node_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });

        assert_eq!(
            before_pages
                .get_page(31)
                .unwrap()
                .as_node(|node: Node<U64Key, &[u8]>| node.as_internal().keys().len()),
            2
        );

        assert_eq!(
            before_pages
                .get_page(1)
                .unwrap()
                .as_node(|node: Node<U64Key, &[u8]>| node.as_internal().keys().get(0)),
            U64Key(5)
        )
    }

    #[test]
    fn delete_with_take_from_right() {
        let before_pages = pages();
        let parent = internal_page_mut(
            &before_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![1, 2, 3, 4],
        );
        let mut node = internal_page_mut(
            &before_pages,
            12,
            vec![U64Key(15), U64Key(17)],
            vec![11, 12, 13],
        );
        let right_sibling = internal_page(
            &before_pages,
            13,
            vec![U64Key(22), U64Key(24), U64Key(26)],
            vec![21, 22, 23, 24],
        );

        node.as_node_mut(|mut node| {
            let mut node = node.as_internal_mut();
            match node.delete(&U64Key(15)).unwrap() {
                InternalDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            };

            match node.rebalance(SiblingsArg::Right(right_sibling)).unwrap() {
                RebalanceResult::TakeFromRight(add_params) => {
                    before_pages.make_shadow(13, 31).unwrap();
                    add_params.take_key_from_right(
                        parent,
                        Some(0),
                        before_pages.mut_page(31).unwrap(),
                    );
                }
                _ => panic!(),
            }
        });

        let aux_pages = pages();
        let node_expected = internal_page(
            &aux_pages,
            12,
            vec![U64Key(17), U64Key(20)],
            vec![11, 13, 21],
        );
        node.as_node(|before| {
            node_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });

        let right_sibling_expected = internal_page(
            &aux_pages,
            13,
            vec![U64Key(24), U64Key(26)],
            vec![22, 23, 24],
        );

        before_pages.get_page(31).unwrap().as_node(|before| {
            right_sibling_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });

        let parent_expected = internal_page(
            &aux_pages,
            1,
            vec![U64Key(10), U64Key(22), U64Key(30)],
            vec![1, 2, 3, 4],
        );

        before_pages.get_page(1).unwrap().as_node(|before| {
            parent_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });
    }

    #[test]
    fn delete_with_left_merge() {
        let before_pages = pages();
        let parent = internal_page_mut(
            &before_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![1, 2, 3, 4],
        );
        let mut node = internal_page_mut(
            &before_pages,
            12,
            vec![U64Key(15), U64Key(17)],
            vec![11, 12, 13],
        );
        let left_sibling = internal_page(
            &before_pages,
            11,
            vec![U64Key(5), U64Key(7)],
            vec![21, 22, 23],
        );

        node.as_node_mut(|mut node| {
            let mut node = node.as_internal_mut();
            match node.delete(&U64Key(15)).unwrap() {
                InternalDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            };

            match node.rebalance(SiblingsArg::Left(left_sibling)).unwrap() {
                RebalanceResult::MergeIntoLeft(add_params) => {
                    before_pages.make_shadow(11, 31).unwrap();
                    add_params
                        .merge_into_left(parent, Some(0), before_pages.mut_page(31).unwrap())
                        .unwrap();
                }
                _ => panic!(),
            }
        });

        assert_eq!(
            node.as_node(|n: Node<U64Key, &[u8]>| n.as_internal().keys().len()),
            0
        );

        let aux_pages = pages();
        let left_sibling_expected = internal_page(
            &aux_pages,
            11,
            vec![U64Key(5), U64Key(7), U64Key(10), U64Key(17)],
            vec![21, 22, 23, 11, 13],
        );

        before_pages.get_page(31).unwrap().as_node(|before| {
            left_sibling_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });

        let parent_expected = internal_page_mut(
            &aux_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![1, 2, 3, 4],
        );

        before_pages.get_page(1).unwrap().as_node(|before| {
            parent_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });
    }

    #[test]
    fn delete_with_right_merge() {
        let before_pages = pages();

        let parent = internal_page_mut(
            &before_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![1, 2, 3, 4],
        );

        let mut node = internal_page_mut(
            &before_pages,
            12,
            vec![U64Key(15), U64Key(17)],
            vec![11, 12, 13],
        );

        let right_sibling = internal_page(
            &before_pages,
            13,
            vec![U64Key(25), U64Key(27)],
            vec![21, 22, 23],
        );

        node.as_node_mut(|mut node| {
            let mut node = node.as_internal_mut();
            match node.delete(&U64Key(15)).unwrap() {
                InternalDeleteStatus::NeedsRebalance => (),
                _ => panic!(),
            };

            match node.rebalance(SiblingsArg::Right(right_sibling)).unwrap() {
                RebalanceResult::MergeIntoSelf(add_params) => {
                    before_pages.make_shadow(13, 33).unwrap();
                    add_params
                        .merge_into_self(parent, Some(0), before_pages.mut_page(33).unwrap())
                        .unwrap();
                }
                _ => panic!(),
            };
        });

        let aux_pages = pages();
        let node_expected = internal_page(
            &aux_pages,
            12,
            vec![U64Key(17), U64Key(20), U64Key(25), U64Key(27)],
            vec![11, 13, 21, 22, 23],
        );

        node.as_node(|before| {
            node_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });

        let parent_expected = internal_page(
            &aux_pages,
            1,
            vec![U64Key(10), U64Key(20), U64Key(30)],
            vec![1, 2, 3, 4],
        );

        before_pages.get_page(1).unwrap().as_node(|before| {
            parent_expected.as_node(|node_expected| {
                assert_eq!(before.as_internal(), node_expected.as_internal())
            })
        });
    }
}
