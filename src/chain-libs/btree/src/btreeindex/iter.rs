use super::version_management::transaction::ReadTransaction;

use super::node::{Node, NodeRef};

use std::borrow::Borrow;

use crate::FixedSize;

use super::PageId;

use std::marker::PhantomData;
use std::ops::{Bound, RangeBounds};

pub struct BTreeIterator<'a, R, Q, K, V>
where
    R: RangeBounds<Q>,
    K: Borrow<Q>,
{
    range: R,
    tx: ReadTransaction<'a>,
    phantom_data: PhantomData<(Q, K, V)>,
    // usually b+trees have pointers between leaves, but doing this in a copy on write tree is not possible (or at least it requires cloning all the leaves at each operation),
    // so we use a stack to keep track of parents
    // the second parameter is used to keep track of what's the next descendant of that node
    stack: Vec<(PageId, usize)>,
    current_position: usize,
    current_leaf: PageId,
}

impl<'a, R, Q, K: FixedSize, V: FixedSize> BTreeIterator<'a, R, Q, K, V>
where
    R: RangeBounds<Q>,
    K: Borrow<Q>,
    Q: Ord,
{
    pub(super) fn new(tx: ReadTransaction<'a>, range: R) -> Self {
        let mut stack = vec![];
        let mut current = tx.get_page(tx.root()).unwrap();

        let (leaf, starting_pos) = match range.start_bound() {
            Bound::Excluded(start) | Bound::Included(start) => {
                // find the starting leaf, and populate the stack with the path leading to it
                // this is the only search needed, as afterwards we just go in-order
                loop {
                    let is_internal = current.as_node(|node: Node<K, &[u8]>| {
                        node.try_as_internal().map(|inode| {
                            let upper_pivot = match inode.keys().binary_search(start) {
                                Ok(pos) => pos + 1,
                                Err(pos) => pos,
                            };

                            let children_len = inode.children().len();

                            let pivot = if upper_pivot < children_len {
                                upper_pivot
                            } else {
                                children_len.checked_sub(1).unwrap()
                            };

                            let new_current_id = inode.children().get(pivot);
                            (new_current_id, pivot)
                        })
                    });

                    if let Some((new_current_id, upper_pivot)) = is_internal {
                        stack.push((current.id(), upper_pivot));
                        current = tx.get_page(new_current_id).unwrap();
                    } else {
                        break current.as_node(|node: Node<K, &[u8]>| {
                            let excluded = match range.start_bound() {
                                Bound::Excluded(_) => true,
                                _ => false,
                            };

                            match node.as_leaf::<V>().keys().binary_search(start) {
                                Ok(pos) => (current.id(), pos + if excluded { 1 } else { 0 }),
                                Err(pos) => (current.id(), pos + 1),
                            }
                        });
                    }
                }
            }
            Bound::Unbounded => {
                let current_position = 0;
                let mut current_leaf = None;
                descend_leftmost::<_, _, K, V>(
                    &tx,
                    tx.root(),
                    |internal_node_page| stack.push((internal_node_page, 0)),
                    |leaf_node_page| current_leaf = Some(leaf_node_page),
                );

                (current_leaf.unwrap(), current_position)
            }
        };

        BTreeIterator {
            tx,
            range,
            stack,
            phantom_data: PhantomData,
            current_position: starting_pos,
            current_leaf: leaf,
        }
    }
}

impl<'a, R, Q, K: FixedSize, V: FixedSize> Iterator for BTreeIterator<'a, R, Q, K, V>
where
    K: Borrow<Q>,
    R: RangeBounds<Q>,
    Q: Ord,
{
    type Item = V;
    fn next(&mut self) -> Option<V> {
        let current_position = self.current_position;

        enum NextStep<T> {
            EndReached,
            InLeaf(T),
            MoveToRightSibling,
        }

        let next = self
            .tx
            .get_page(self.current_leaf)
            .unwrap()
            .as_node(|node: Node<K, &[u8]>| {
                match node.as_leaf::<V>().keys().try_get(current_position) {
                    None => NextStep::MoveToRightSibling,
                    Some(key) => {
                        let is_in_bounds = match self.range.end_bound() {
                            Bound::Included(end) => key.borrow().borrow() <= end,
                            Bound::Excluded(end) => key.borrow().borrow() < end,
                            Bound::Unbounded => true,
                        };
                        if is_in_bounds {
                            NextStep::InLeaf(
                                node.as_leaf::<V>()
                                    .values()
                                    .try_get(current_position)
                                    .map(|v| v.borrow().clone())
                                    .unwrap(),
                            )
                        } else {
                            NextStep::EndReached
                        }
                    }
                }
            });

        match next {
            NextStep::InLeaf(v) => {
                self.current_position += 1;
                Some(v)
            }
            NextStep::EndReached => None,
            NextStep::MoveToRightSibling => {
                while let Some((internal_node, last_position)) = self.stack.pop() {
                    let next = last_position + 1;
                    if let Some(child) = self
                        .tx
                        .get_page(internal_node)
                        .unwrap()
                        .as_node(|node: Node<K, &[u8]>| node.as_internal().children().try_get(next))
                    {
                        self.stack.push((internal_node, next));
                        let stack = &mut self.stack;

                        let current_leaf = &mut self.current_leaf;
                        let current_position = &mut self.current_position;
                        descend_leftmost::<_, _, K, V>(
                            &self.tx,
                            child,
                            |internal_node_page| {
                                stack.push((internal_node_page, 0));
                            },
                            |leaf_node_page| {
                                *current_leaf = leaf_node_page;
                                *current_position = 0;
                            },
                        );
                        return self.next();
                    }
                }

                None
            }
        }
    }
}

fn descend_leftmost<'a, I, L, K, V>(
    tx: &'a ReadTransaction<'a>,
    starting_node: PageId,
    mut on_internal: I,
    on_leaf: L,
) where
    I: FnMut(PageId),
    L: FnOnce(PageId),
    K: FixedSize,
    V: FixedSize,
{
    let mut current = tx.get_page(starting_node).unwrap();
    loop {
        let next = current.as_node(|node: Node<K, &[u8]>| {
            node.try_as_internal().map(|inode| {
                on_internal(current.id());
                inode.children().get(0)
            })
        });

        if let Some(new_current_id) = next {
            current = tx.get_page(new_current_id).unwrap();
        } else {
            on_leaf(current.id());
            return;
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::btreeindex::tests::new_tree;

    use crate::tests::U64Key;

    #[test]
    fn range_query_empty_tree() {
        let tree = new_tree();

        let a = 10u64;
        let b = 11u64;
        let mut found = tree.range(U64Key(a)..U64Key(b));
        assert!(found.next().is_none());
    }

    #[quickcheck]
    fn qc_range_query_included_excluded(a: u64, b: u64) -> bool {
        let tree = new_tree();
        let n: u64 = 2000;

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        let found: Vec<_> = tree.range(U64Key(a)..U64Key(b)).collect();
        let expected: Vec<_> = (a..std::cmp::min(b, n)).collect();

        found == expected
    }

    #[quickcheck]
    fn qc_range_query_included_included(a: u64, b: u64) -> bool {
        let tree = new_tree();
        let n: u64 = 2000;

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        let found: Vec<_> = tree.range(U64Key(a)..=U64Key(b)).collect();
        let expected: Vec<_> = (a..=std::cmp::min(b, n)).collect();

        found == expected
    }

    #[test]
    fn range_query_unbounded() {
        let tree = new_tree();
        let n: u64 = 2000;

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        let found: Vec<_> = tree.range(..).collect();

        assert_eq!(found, (0..n).collect::<Vec<_>>());
    }
}
