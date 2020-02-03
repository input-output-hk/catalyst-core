use super::Node;
use crate::btreeindex::{Children, ChildrenMut, Keys, KeysMut, PageId};
use crate::Key;
use crate::MemPage;
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
    key_buffer_size: usize,
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

impl<'b, K, T> InternalNode<'b, K, T>
where
    K: Key,
    T: AsRef<[u8]> + AsMut<[u8]> + 'b,
{
    /// Init the given slice (mutating it) so it is a valid (empty) InternalNode that
    /// can be later read with `from_raw`
    pub fn init(key_buffer_size: usize, buffer: T) -> InternalNode<'b, K, T> {
        let mut uninit = Self::from_raw(key_buffer_size, buffer);
        uninit.set_len(0);
        uninit
    }

    /// mutable version of node interpretated over the given slice
    /// this shouldn't be called before calling `init`
    // TODO: add more rigorous type checking?
    pub fn from_raw(key_buffer_size: usize, data: T) -> InternalNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);
        assert!(data.as_ref().len() > 0);

        let size_per_key = key_buffer_size + size_of::<PageId>();
        let extra_size = LEN_SIZE - LEN_START;

        let max_keys = (usize::try_from(data.as_ref().len()).unwrap()
            - usize::try_from(extra_size).unwrap()
            - size_of::<PageId>())
            / size_per_key;

        InternalNode {
            max_keys,
            key_buffer_size,
            data,
            phantom: PhantomData,
        }
    }

    // The first insertion of an InternalNode is different, because in general we can insert
    // only new keys and the right child. When the node has only one key, we need two children so
    // the first insertion must insert two keys
    pub fn insert_first<'me>(&'me mut self, key: K, left: PageId, right: PageId) {
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
    pub fn insert<'me>(
        &'me mut self,
        key: K,
        node_id: PageId,
        allocate: impl FnMut() -> Node<K, MemPage>,
    ) -> InternalInsertStatus<K> {
        // Non empty, maybe encapsulate in some kind of state machine
        assert!(self.keys().len() > 0);
        match self.keys().binary_search(&key) {
            Ok(_) => InternalInsertStatus::DuplicatedKey(key),
            Err(index) => self.insert_key_child(index.try_into().unwrap(), key, node_id, allocate),
        }
    }

    pub fn children_mut<'me>(&'me mut self) -> ChildrenMut<'me> {
        let len = if self.keys().len() > 0 {
            self.keys().len().checked_add(1).unwrap() as usize
        } else {
            0
        };

        let base = LEN_SIZE + (self.max_keys * self.key_buffer_size);
        let data = &mut self.data.as_mut()
            [base..base + (self.max_keys.checked_add(1).unwrap()) * size_of::<PageId>()];

        ChildrenMut::new_static_size(data, len)
    }

    fn keys_mut<'me>(&'me mut self) -> KeysMut<'me, K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);

        let data =
            &mut self.data.as_mut()[LEN_SIZE..LEN_SIZE + self.max_keys * self.key_buffer_size];

        KeysMut::new_dynamic_size(data, len.try_into().unwrap(), self.key_buffer_size)
    }

    fn insert_key_child<'me>(
        &'me mut self,
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
                if pos < m.try_into().unwrap() {
                    let mut right_node_internal = right_node.as_internal_mut().unwrap();
                    let split_key = self.keys().get(m - 1 as usize).unwrap().borrow().clone();

                    let mut keys_mut = right_node_internal.keys_mut();
                    for k in self.keys().sub(m..self.keys().len()).into_iter() {
                        keys_mut.append(k.borrow()).unwrap();
                    }

                    let new_len = keys_mut.len();

                    let mut children_mut = right_node_internal.children_mut();

                    for c in self
                        .children()
                        .sub(m as usize..self.children().len())
                        .into_iter()
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
                // the key would be inserted in the last half
                else if pos > m.try_into().unwrap() {
                    let mut right_internal_node = right_node.as_internal_mut().unwrap();
                    let split_key = self.keys().get(m as usize).unwrap().borrow().clone();

                    let mut keys_mut = right_internal_node.keys_mut();
                    for k in self.keys().sub(m + 1..pos as usize).into_iter() {
                        keys_mut.append(k.borrow()).unwrap();
                    }

                    keys_mut.append(&key).unwrap();

                    for k in self.keys().sub(pos as usize..self.keys().len()).into_iter() {
                        keys_mut.append(k.borrow()).unwrap();
                    }

                    let new_len = keys_mut.len();
                    let mut children_mut = right_internal_node.children_mut();

                    for c in self
                        .children()
                        .sub(m as usize + 1..pos as usize + 1)
                        .into_iter()
                        .chain(Some(node_id).into_iter())
                        .chain(
                            self.children()
                                .sub(pos as usize + 1..self.children().len())
                                .into_iter(),
                        )
                    {
                        children_mut.append(c.borrow()).unwrap();
                    }

                    right_internal_node.set_len(new_len);

                    // Truncate node to have m elements
                    self.set_len(m as usize);
                    InternalInsertStatus::Split(split_key.clone(), right_node)
                } else {
                    // pos == m
                    let mut right_internal_node = right_node.as_internal_mut().unwrap();

                    let split_key = key;

                    let mut keys_mut = right_internal_node.keys_mut();
                    for k in self.keys().sub(m as usize..self.keys().len()).into_iter() {
                        keys_mut.append(k.borrow()).unwrap();
                    }

                    let new_len = keys_mut.len();
                    let mut children_mut = right_internal_node.children_mut();

                    for c in Some(node_id).into_iter().chain(
                        self.children()
                            .sub(m as usize + 1..self.children().len())
                            .into_iter(),
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

    fn set_len(&mut self, new_len: usize) {
        let new_len = u64::try_from(new_len).unwrap();
        LittleEndian::write_u64(&mut self.data.as_mut()[0..LEN_SIZE], new_len);
    }
}

impl<'b, K, T> InternalNode<'b, K, T>
where
    K: Key,
    T: AsRef<[u8]> + 'b,
{
    pub fn view(key_buffer_size: usize, data: T) -> InternalNode<'b, K, T> {
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<PageId>()), 0);
        assert_eq!(data.as_ref().as_ptr().align_offset(size_of::<u64>()), 0);
        assert!(data.as_ref().len() > 0);

        let size_per_key = key_buffer_size + size_of::<PageId>();
        let extra_size = LEN_SIZE - LEN_START;

        let max_keys = (usize::try_from(data.as_ref().len()).unwrap()
            - usize::try_from(extra_size).unwrap()
            - size_of::<PageId>())
            / size_per_key;

        InternalNode {
            max_keys,
            key_buffer_size,
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

    pub(crate) fn children<'me>(&'me self) -> Children<'me> {
        let len = if self.keys().len() > 0 {
            self.keys().len().checked_add(1).unwrap() as usize
        } else {
            0
        };

        let base = LEN_SIZE + (self.max_keys * self.key_buffer_size);
        let data = &self.data.as_ref()
            [base..base + (self.max_keys.checked_add(1).unwrap()) * size_of::<PageId>()];

        Children::new_static_size(data, len)
    }

    pub(crate) fn keys<'me>(&'me self) -> Keys<'me, K> {
        let len = LittleEndian::read_u64(&self.data.as_ref()[0..LEN_SIZE]);

        let data = &self.data.as_ref()[LEN_SIZE..LEN_SIZE + self.max_keys * self.key_buffer_size];

        Keys::new_dynamic_size(data, len.try_into().unwrap(), self.key_buffer_size)
    }
}
