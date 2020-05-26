pub mod internal_node;
pub mod leaf_node;

use marker::*;
use std::marker::PhantomData;

use crate::FixedSize;
pub(crate) use internal_node::{InternalInsertStatus, InternalNode};
pub(crate) use leaf_node::{LeafInsertStatus, LeafNode};

const LEN_SIZE: usize = 8;
const TAG_SIZE: usize = 8;

pub struct Node<K, T> {
    data: T,
    phantom: PhantomData<[K]>,
}

/// Trait used to abstract over the input for the rebalance algorithm, taking a Node<K, &[u8]> could be enough in normal cases, but this allows things like RAIIGuards
pub trait NodeRef {
    fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: FixedSize;
}

/// Trait used to abstract over the input for the rebalance algorithm, taking a Node<K, &mut [u8]> could be enough in normal cases, but this allows things like RAIIGuards
pub trait NodeRefMut: NodeRef {
    fn as_node_mut<K, R>(&mut self, f: impl FnOnce(Node<K, &mut [u8]>) -> R) -> R
    where
        K: FixedSize;
}

pub(crate) enum NodeTag {
    Internal = 0,
    Leaf = 1,
}

/// Demultiplexer for the
pub enum RebalanceResult<NodeType> {
    TakeFromLeft(RebalanceSiblingArg<TakeFromLeft, NodeType>),
    TakeFromRight(RebalanceSiblingArg<TakeFromRight, NodeType>),
    MergeIntoLeft(RebalanceSiblingArg<MergeIntoLeft, NodeType>),
    MergeIntoSelf(RebalanceSiblingArg<MergeIntoSelf, NodeType>),
}

/// Auxiliary type for the rebalance process. After calling rebalance, an instance of this if returned with the proper bounds and only one function for providing the
/// required arguments for the given strategy. Note: This is not enforced by anything, is just a convention.

// The type is necessary in order to only clone/shadow nodes only when it's certain they will be used. For example, if both siblings of the rebalancing node have spare keys,
// then the selected strategy will always be to take from the left, this means that only the left sibling needs to be mutated/cloned. Doing the cloning lazily with closures
// would be probably more natural, but I find this approach simpler from a borrowing/state/generic bounds management perspective.
pub struct RebalanceSiblingArg<Strategy, NodeType> {
    // in practice, this would be a type that somehow borrows a Node, in order to operate on it, but there is no Node trait, so it is unbounded
    node: NodeType,
    phantom: PhantomData<Strategy>,
}

impl<Strategy, NodeType> RebalanceSiblingArg<Strategy, NodeType> {
    fn new(node: NodeType) -> Self {
        Self {
            node,
            phantom: PhantomData,
        }
    }
}

mod marker {
    pub struct TakeFromLeft;
    pub struct TakeFromRight;
    pub struct MergeIntoLeft;
    pub struct MergeIntoSelf;
}

/// input for the rebalance algorithms, this is used to define which of the strategies is used
/// in both cases (leaf and internal) these are:
/// steal key from left sibling, steal a key from right sibling, merge current into left sibling, merge right sibling into current.
// the generic is bound only on NodeRef (immutable) because this is only used to ask the sibling if it has keys to borrow
pub enum SiblingsArg<N: NodeRef> {
    Left(N),
    Right(N),
    Both(N, N),
}

impl<N: NodeRef> SiblingsArg<N> {
    pub fn new_from_options(left_sibling: Option<N>, right_sibling: Option<N>) -> Self {
        match (left_sibling, right_sibling) {
            (Some(left), Some(right)) => SiblingsArg::Both(left, right),
            (Some(left), None) => SiblingsArg::Left(left),
            (None, Some(right)) => SiblingsArg::Right(right),
            (None, None) => unreachable!(),
        }
    }
}

impl<'b, K, T> Node<K, T>
where
    K: FixedSize,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    pub(crate) fn new_internal(buffer: T) -> Node<K, T> {
        let mut buffer = buffer;
        buffer.as_mut()[0..TAG_SIZE].copy_from_slice(&0u64.to_le_bytes());
        InternalNode::<K, &mut [u8]>::init(&mut buffer.as_mut()[8..]);
        Node {
            data: buffer,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new_leaf<V: FixedSize>(buffer: T) -> Node<K, T> {
        let mut buffer = buffer;
        buffer.as_mut()[0..TAG_SIZE].copy_from_slice(&1u64.to_le_bytes());
        LeafNode::<K, V, &mut [u8]>::init(&mut buffer.as_mut()[8..]);
        Node {
            data: buffer,
            phantom: PhantomData,
        }
    }

    pub(crate) fn try_as_internal_mut<'i: 'b>(
        &'i mut self,
    ) -> Option<InternalNode<'b, K, &mut [u8]>> {
        // the unsafe part is actually in Node::from_raw, so at this point we don't care that much
        match self.get_tag() {
            NodeTag::Internal => unsafe {
                Some(InternalNode::from_raw(&mut self.data.as_mut()[TAG_SIZE..]))
            },
            NodeTag::Leaf => None,
        }
    }

    pub(crate) fn try_as_leaf_mut<'i: 'b, V: FixedSize>(
        &'i mut self,
    ) -> Option<LeafNode<'b, K, V, &mut [u8]>> {
        // the unsafe part is actually in Node::from_raw, so at this point we don't care that much
        match self.get_tag() {
            NodeTag::Leaf => unsafe {
                Some(LeafNode::from_raw(&mut self.data.as_mut()[TAG_SIZE..]))
            },
            NodeTag::Internal => None,
        }
    }

    pub(crate) fn as_internal_mut(&mut self) -> InternalNode<K, &mut [u8]> {
        self.try_as_internal_mut().unwrap()
    }

    pub(crate) fn as_leaf_mut<V: FixedSize>(&mut self) -> LeafNode<K, V, &mut [u8]> {
        self.try_as_leaf_mut().unwrap()
    }
}

impl<'b, K, T> Node<K, T>
where
    K: FixedSize,
    T: AsRef<[u8]> + 'b,
{
    pub(crate) unsafe fn from_raw(data: T) -> Node<K, T> {
        Node {
            data,
            phantom: PhantomData,
        }
    }

    pub(crate) fn get_tag(&self) -> NodeTag {
        let mut bytes = [0u8; LEN_SIZE];
        bytes.copy_from_slice(&self.data.as_ref()[..LEN_SIZE]);
        match u64::from_le_bytes(bytes) {
            0 => NodeTag::Internal,
            1 => NodeTag::Leaf,
            _ => unreachable!(),
        }
    }

    pub(crate) fn try_as_internal<'i: 'b>(&'i self) -> Option<InternalNode<'b, K, &[u8]>> {
        match self.get_tag() {
            NodeTag::Internal => Some(InternalNode::view(&self.data.as_ref()[LEN_SIZE..])),
            NodeTag::Leaf => None,
        }
    }

    pub(crate) fn try_as_leaf<'i: 'b, V: FixedSize>(&'i self) -> Option<LeafNode<'b, K, V, &[u8]>> {
        match self.get_tag() {
            NodeTag::Leaf => Some(LeafNode::view(&self.data.as_ref()[LEN_SIZE..])),
            NodeTag::Internal => None,
        }
    }

    pub(crate) fn as_leaf<V: FixedSize>(&self) -> LeafNode<K, V, &[u8]> {
        self.try_as_leaf().unwrap()
    }

    pub(crate) fn as_internal(&self) -> InternalNode<K, &[u8]> {
        self.try_as_internal().unwrap()
    }
}

impl<'b, K> Node<K, crate::mem_page::MemPage>
where
    K: FixedSize,
{
    pub(crate) fn into_page(self) -> crate::mem_page::MemPage {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btreeindex::pages::{
        borrow::{Immutable, Mutable},
        PageHandle, Pages, PagesInitializationParams,
    };
    use crate::btreeindex::PageId;
    use crate::mem_page::MemPage;
    use crate::storage::MmapStorage;
    use crate::tests::U64Key;
    use std::mem::size_of;
    use tempfile::tempfile;

    pub const PAGE_SIZE: u64 = (1 << 20) * 128; // 128mb

    pub fn pages() -> Pages {
        let page_size = 8 + 8 + 3 * size_of::<U64Key>() + 5 * size_of::<PageId>() + 4 + 8;
        let storage = MmapStorage::new(tempfile().unwrap(), PAGE_SIZE).unwrap();
        let params = PagesInitializationParams {
            storage,
            page_size: page_size as u16,
        };

        Pages::new(params)
    }

    pub fn allocate_internal() -> Node<U64Key, MemPage> {
        let page_size = 8 + 8 + 3 * size_of::<U64Key>() + 4 * size_of::<PageId>();
        let page = MemPage::new(page_size);
        Node::new_internal(page)
    }

    pub fn internal_page_mut(
        pages: &Pages,
        page_id: PageId,
        keys: Vec<U64Key>,
        children: Vec<u32>,
    ) -> PageHandle<Mutable> {
        assert_eq!(keys.len() + 1, children.len());

        let mut page = pages.mut_page(page_id).unwrap();

        page.as_slice(|slice| {
            InternalNode::<U64Key, &mut [u8]>::init(slice);
        });

        page.as_node_mut(|mut node| {
            let mut iter = keys.iter();

            if let Some(first_key) = iter.next() {
                node.as_internal_mut()
                    .insert_first((*first_key).clone(), children[0], children[1]);
            }

            for (k, c) in iter.zip(children[2..].iter()) {
                match node
                    .as_internal_mut()
                    .insert((*k).clone(), *c, &mut allocate_internal)
                {
                    InternalInsertStatus::Ok => (),
                    _ => panic!("insertion shouldn't split"),
                };
            }
        });

        page
    }

    pub fn internal_page(
        pages: &Pages,
        page_id: PageId,
        keys: Vec<U64Key>,
        children: Vec<u32>,
    ) -> PageHandle<Immutable> {
        assert_eq!(keys.len() + 1, children.len());

        {
            internal_page_mut(pages, page_id, keys, children);
        }

        pages.get_page(page_id).unwrap()
    }

    #[test]
    fn insert_internal_with_split_at_first() {
        let insertions = [2u32, 3, 1];
        let mem_size = 8 + 8 + 2 * 8 + 3 * 4;
        internal_insert_with_split(mem_size, &insertions);
    }

    #[test]
    fn insert_internal_with_split_at_middle() {
        let insertions = [1, 2, 3];
        let mem_size = 8 + 8 + 2 * 8 + 3 * 4;
        internal_insert_with_split(mem_size, &insertions);
    }

    #[test]
    fn insert_internal_with_split_at_last() {
        let insertions = [1, 3, 2];
        let mem_size = 8 + 8 + 2 * 8 + 3 * 4;
        internal_insert_with_split(mem_size, &insertions);
    }

    fn internal_insert_with_split(mem_size: usize, insertions: &[u32]) {
        let i1 = insertions[0];
        let i2 = insertions[1];
        let i3 = insertions[2];

        let buffer = MemPage::new(mem_size);
        buffer.as_ref().len();
        let mut node: Node<U64Key, MemPage> = Node::new_internal(buffer);

        let mut allocate = || {
            let page = MemPage::new(mem_size);
            Node::new_internal(page)
        };

        node.as_internal_mut()
            .insert_first(U64Key(i1 as u64), 0u32, i1);
        match node
            .as_internal_mut()
            .insert(U64Key(i2 as u64), i2, &mut allocate)
        {
            InternalInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };

        match node
            .as_internal_mut()
            .insert(U64Key(i3 as u64), i3, &mut allocate)
        {
            InternalInsertStatus::Split(U64Key(2), new_node) => {
                assert_eq!(new_node.as_internal().keys().len(), 1);
                assert_eq!(new_node.as_internal().keys().get(0), U64Key(3));
                assert_eq!(new_node.as_internal().children().len(), 2);
                assert_eq!(new_node.as_internal().children().get(0), 2);
                assert_eq!(new_node.as_internal().children().get(1), 3);
            }
            _ => {
                panic!("third insertion should split");
            }
        };

        assert_eq!(node.as_internal().keys().len(), 1);
        assert_eq!(node.as_internal().keys().get(0), U64Key(1));
        assert_eq!(node.as_internal().children().len(), 2);
        assert_eq!(node.as_internal().children().get(0), 0u32);
        assert_eq!(node.as_internal().children().get(1), 1u32);
    }

    #[test]
    fn insert_leaf_with_split_at_first() {
        let insertions = [2, 3, 1];
        let mem_size = 8usize + 8 + 2usize * size_of::<PageId>() + 3 * 12;
        leaf_insert_with_split(mem_size, &insertions);
    }

    #[test]
    fn insert_leaf_with_split_at_middle() {
        let insertions = [1, 2, 3];
        let mem_size = 8usize + 8 + 2usize * size_of::<PageId>() + 3 * 12;
        leaf_insert_with_split(mem_size, &insertions);
    }

    #[test]
    fn insert_leaf_with_split_at_last() {
        let insertions = [1, 3, 2];
        let mem_size = 8usize + 8 + 2usize * size_of::<PageId>() + 3 * 12;
        leaf_insert_with_split(mem_size, &insertions);
    }

    fn leaf_insert_with_split(mem_size: usize, insertions: &[u64]) {
        let i1 = insertions[0];
        let i2 = insertions[1];
        let i3 = insertions[2];

        let buffer = MemPage::new(mem_size);
        let mut node: Node<U64Key, MemPage> = Node::new_leaf::<u64>(buffer);

        let mut allocate = || {
            let page = MemPage::new(mem_size);
            Node::new_leaf::<u64>(page)
        };

        match node.as_leaf_mut().insert(U64Key(i1), i1, &mut allocate) {
            LeafInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };
        match node.as_leaf_mut().insert(U64Key(i2), i2, &mut allocate) {
            LeafInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };
        match node.as_leaf_mut().insert(U64Key(i3), i3, &mut allocate) {
            LeafInsertStatus::Split(U64Key(2), new_node) => {
                let new_leaf = new_node.as_leaf::<u64>();
                assert_eq!(new_leaf.keys().len(), 2);
                assert_eq!(new_leaf.keys().get(0), U64Key(2));
                assert_eq!(new_leaf.keys().get(1), U64Key(3));
                assert_eq!(new_leaf.values().len(), 2);
                assert_eq!(new_leaf.values().get(0), 2);
                assert_eq!(new_leaf.values().get(1), 3);
            }
            _ => {
                panic!("third insertion should split");
            }
        };

        assert_eq!(node.as_leaf::<u64>().keys().len(), 1);
        assert_eq!(node.as_leaf::<u64>().keys().get(0), U64Key(1));
        assert_eq!(node.as_leaf::<u64>().values().len(), 1);
        assert_eq!(node.as_leaf::<u64>().values().get(0), 1);
    }
}
