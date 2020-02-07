pub mod internal_node;
pub mod leaf_node;

use std::marker::PhantomData;

use crate::Key;
pub(crate) use internal_node::{InternalInsertStatus, InternalNode};
pub(crate) use leaf_node::{LeafInsertStatus, LeafNode};

const LEN_SIZE: usize = 8;
const TAG_SIZE: usize = 8;

pub struct Node<K, T> {
    data: T,
    key_buffer_size: usize,
    phantom: PhantomData<[K]>,
}

pub(crate) enum NodeTag {
    Internal = 0,
    Leaf = 1,
}

impl<'b, K, T> Node<K, T>
where
    K: Key,
    T: AsMut<[u8]> + AsRef<[u8]> + 'b,
{
    pub(crate) fn from_raw_mut(data: T, key_buffer_size: usize) -> Node<K, T> {
        Node {
            data,
            key_buffer_size,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new_internal(key_buffer_size: usize, buffer: T) -> Node<K, T> {
        let mut buffer = buffer;
        buffer.as_mut()[0..TAG_SIZE].copy_from_slice(&0u64.to_le_bytes());
        InternalNode::<K, &mut [u8]>::init(key_buffer_size, &mut buffer.as_mut()[8..]);
        Node {
            data: buffer,
            key_buffer_size,
            phantom: PhantomData,
        }
    }

    pub(crate) fn new_leaf(key_buffer_size: usize, buffer: T) -> Node<K, T> {
        let mut buffer = buffer;
        buffer.as_mut()[0..TAG_SIZE].copy_from_slice(&1u64.to_le_bytes());
        LeafNode::<K, &mut [u8]>::init(key_buffer_size, &mut buffer.as_mut()[8..]);
        Node {
            data: buffer,
            key_buffer_size,
            phantom: PhantomData,
        }
    }

    pub(crate) fn as_internal_mut<'i: 'b>(&'i mut self) -> Option<InternalNode<'b, K, &mut [u8]>> {
        match self.get_tag() {
            NodeTag::Internal => Some(InternalNode::from_raw(
                self.key_buffer_size,
                &mut self.data.as_mut()[TAG_SIZE..],
            )),
            NodeTag::Leaf => None,
        }
    }

    pub(crate) fn as_leaf_mut<'i: 'b>(&'i mut self) -> Option<LeafNode<'b, K, &mut [u8]>> {
        match self.get_tag() {
            NodeTag::Leaf => Some(LeafNode::from_raw(
                self.key_buffer_size,
                &mut self.data.as_mut()[TAG_SIZE..],
            )),
            NodeTag::Internal => None,
        }
    }
}

impl<'b, K, T> Node<K, T>
where
    K: Key,
    T: AsRef<[u8]> + 'b,
{
    pub(crate) fn from_raw(data: T, key_buffer_size: usize) -> Node<K, T> {
        Node {
            data,
            key_buffer_size,
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

    pub(crate) fn as_internal<'i: 'b>(&'i self) -> Option<InternalNode<'b, K, &[u8]>> {
        match self.get_tag() {
            NodeTag::Internal => Some(InternalNode::view(
                self.key_buffer_size,
                &self.data.as_ref()[LEN_SIZE..],
            )),
            NodeTag::Leaf => None,
        }
    }

    pub(crate) fn as_leaf<'i: 'b>(&'i self) -> Option<LeafNode<'b, K, &[u8]>> {
        match self.get_tag() {
            NodeTag::Leaf => Some(LeafNode::view(
                self.key_buffer_size,
                &self.data.as_ref()[LEN_SIZE..],
            )),
            NodeTag::Internal => None,
        }
    }
}

impl<'b, K> Node<K, crate::mem_page::MemPage>
where
    K: Key,
{
    pub(crate) fn to_page(self) -> crate::mem_page::MemPage {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btreeindex::PageId;
    use crate::mem_page::MemPage;
    use crate::tests::U64Key;
    use std::mem::size_of;

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

        let buffer = MemPage::new(dbg!(mem_size));
        buffer.as_ref().len();
        let mut node: Node<U64Key, MemPage> =
            Node::new_internal(std::mem::size_of::<U64Key>(), buffer);

        let mut allocate = || {
            let page = MemPage::new(mem_size);
            Node::new_internal(std::mem::size_of::<U64Key>(), page)
        };

        node.as_internal_mut()
            .unwrap()
            .insert_first(U64Key(i1 as u64), 0u32, i1);
        match node
            .as_internal_mut()
            .unwrap()
            .insert(U64Key(i2 as u64), i2, &mut allocate)
        {
            InternalInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };

        println!("Inserting splitting key");
        match node
            .as_internal_mut()
            .unwrap()
            .insert(U64Key(i3 as u64), i3, &mut allocate)
        {
            InternalInsertStatus::Split(U64Key(2), new_node) => {
                assert_eq!(new_node.as_internal().unwrap().keys().len(), 1);
                assert_eq!(
                    new_node
                        .as_internal()
                        .unwrap()
                        .keys()
                        .get(0)
                        .expect("Couldn't get first key"),
                    U64Key(3)
                );
                assert_eq!(new_node.as_internal().unwrap().children().len(), 2);
                assert_eq!(
                    new_node
                        .as_internal()
                        .unwrap()
                        .children()
                        .get(0)
                        .expect("Couldn't get first key"),
                    2
                );
                assert_eq!(
                    new_node
                        .as_internal()
                        .unwrap()
                        .children()
                        .get(1)
                        .expect("Couldn't get second key"),
                    3
                );
            }
            _ => {
                panic!("third insertion should split");
            }
        };

        assert_eq!(node.as_internal().unwrap().keys().len(), 1);
        assert_eq!(
            node.as_internal().unwrap().keys().get(0).unwrap(),
            U64Key(1)
        );
        assert_eq!(node.as_internal().unwrap().children().len(), 2);
        assert_eq!(node.as_internal().unwrap().children().get(0).unwrap(), 0u32);
        assert_eq!(node.as_internal().unwrap().children().get(1).unwrap(), 1u32);
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
        let mut node: Node<U64Key, MemPage> = Node::new_leaf(std::mem::size_of::<U64Key>(), buffer);

        let mut allocate = || {
            let page = MemPage::new(mem_size);
            Node::new_leaf(std::mem::size_of::<U64Key>(), page)
        };

        match node
            .as_leaf_mut()
            .unwrap()
            .insert(U64Key(i1), i1, &mut allocate)
        {
            LeafInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };
        match node
            .as_leaf_mut()
            .unwrap()
            .insert(U64Key(i2), i2, &mut allocate)
        {
            LeafInsertStatus::Ok => (),
            _ => panic!("second insertion shouldn't split"),
        };
        match node
            .as_leaf_mut()
            .unwrap()
            .insert(U64Key(i3), i3, &mut allocate)
        {
            LeafInsertStatus::Split(U64Key(2), new_node) => {
                let new_leaf = new_node.as_leaf().unwrap();
                assert_eq!(new_leaf.keys().len(), 2);
                assert_eq!(new_leaf.keys().get(0).unwrap(), U64Key(2));
                assert_eq!(new_leaf.keys().get(1).unwrap(), U64Key(3));
                assert_eq!(new_leaf.values().len(), 2);
                assert_eq!(new_leaf.values().get(0).unwrap(), 2);
                assert_eq!(new_leaf.values().get(1).unwrap(), 3);
            }
            _ => {
                panic!("third insertion should split");
            }
        };

        assert_eq!(node.as_leaf().unwrap().keys().len(), 1);
        assert_eq!(node.as_leaf().unwrap().keys().get(0).unwrap(), U64Key(1));
        assert_eq!(node.as_leaf().unwrap().values().len(), 1);
        assert_eq!(node.as_leaf().unwrap().values().get(0).unwrap(), 1);
    }
}
