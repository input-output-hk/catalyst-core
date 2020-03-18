mod metadata;
// FIXME: allow dead code momentarily, because all of the delete algorithms are unused, and placing the directive with more granularity would be too troublesome
#[allow(dead_code)]
mod node;
mod page_manager;
mod pages;
mod version_management;

use version_management::transaction::{InsertTransaction, PageRefMut, ReadTransaction};
use version_management::*;

use crate::mem_page::MemPage;
use crate::BTreeStoreError;
use metadata::{Metadata, StaticSettings};
use node::{InternalInsertStatus, LeafInsertStatus, Node, NodeRef, NodeRefMut};
use pages::{borrow, PageHandle, Pages, PagesInitializationParams};
use std::borrow::Borrow;

use crate::{Key, Value};

use parking_lot::RwLock;
use std::convert::{TryFrom, TryInto};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;

pub type PageId = u32;

pub struct BTree<K> {
    // The metadata file contains the latests confirmed version of the tree
    // this is, the root node, and the list of free pages
    metadata: Mutex<(Metadata, File)>,
    static_settings: StaticSettings,
    pages: RwLock<Pages>,
    transaction_manager: TransactionManager,
    phantom_keys: PhantomData<[K]>,
}

/// Views over continous arrays of data. The buffer represents the total capacity
/// but they keep track of the current actual length of items
use crate::arrayview::ArrayView;
pub(crate) type Children<'a> = ArrayView<'a, &'a [u8], PageId>;
pub(crate) type ChildrenMut<'a> = ArrayView<'a, &'a mut [u8], PageId>;
pub(crate) type Values<'a> = ArrayView<'a, &'a [u8], Value>;
pub(crate) type ValuesMut<'a> = ArrayView<'a, &'a mut [u8], Value>;
pub(crate) type Keys<'a, K> = ArrayView<'a, &'a [u8], K>;
pub(crate) type KeysMut<'a, K> = ArrayView<'a, &'a mut [u8], K>;

impl<'me, K: 'me> BTree<K>
where
    K: Key,
{
    // TODO: add a builder with defaults?
    pub fn new(
        metadata_file: File,
        tree_file: File,
        mut static_settings_file: File,
        page_size: u16,
        key_buffer_size: u32,
    ) -> Result<BTree<K>, BTreeStoreError> {
        let mut metadata = Metadata::new();

        let pages_storage = crate::storage::MmapStorage::new(tree_file)?;

        let mut pages = Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size: page_size.try_into().unwrap(),
            key_buffer_size,
        });

        let first_page_id = metadata.page_manager.new_id();

        let mut root_page = match pages.mut_page(first_page_id) {
            Ok(page) => page,
            Err(_) => {
                pages.extend(first_page_id)?;
                // this is infallible now
                pages.mut_page(first_page_id).unwrap()
            }
        };

        root_page.as_slice(|page| {
            Node::<K, &mut [u8]>::new_leaf(key_buffer_size.try_into().unwrap(), page);
        });

        metadata.set_root(first_page_id);

        let static_settings = StaticSettings {
            page_size,
            key_buffer_size,
        };

        static_settings.write(&mut static_settings_file)?;

        let transaction_manager = TransactionManager::new(&metadata);

        Ok(BTree {
            metadata: Mutex::new((metadata, metadata_file)),
            pages: RwLock::new(pages),
            static_settings,
            transaction_manager,
            phantom_keys: PhantomData,
        })
    }

    pub fn open(
        metadata_file: impl AsRef<Path>,
        tree_file: impl AsRef<Path>,
        static_settings_file: impl AsRef<Path>,
    ) -> Result<BTree<K>, BTreeStoreError> {
        let tree_file = OpenOptions::new().write(true).read(true).open(tree_file)?;
        let pages_storage = crate::storage::MmapStorage::new(tree_file)?;

        let mut static_settings_file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(static_settings_file)?;

        let mut metadata_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(metadata_file)?;

        let metadata = Metadata::read(&mut metadata_file)?;

        let static_settings = StaticSettings::read(&mut static_settings_file)?;

        let pages = RwLock::new(Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size: static_settings.page_size,
            key_buffer_size: static_settings.key_buffer_size,
        }));

        let transaction_manager = TransactionManager::new(&metadata);

        Ok(BTree {
            metadata: Mutex::new((metadata, metadata_file)),
            pages,
            static_settings,
            transaction_manager,
            phantom_keys: PhantomData,
        })
    }

    // sync files to disk and collect old transactions pages
    pub(crate) fn checkpoint(&self) -> Result<(), BTreeStoreError> {
        if let Some(checkpoint) = self.transaction_manager.collect_pending() {
            let new_metadata = checkpoint.new_metadata;

            self.pages.read().sync_file()?;

            let mut guard = self.metadata.lock().unwrap();
            let (_metadata, metadata_file) = &mut *guard;

            metadata_file.seek(SeekFrom::Start(0))?;

            new_metadata.write(metadata_file)?;
            metadata_file.sync_all()?;

            // this part is not actually important
            guard.0 = new_metadata;
        }
        Ok(())
    }

    pub fn insert_async(&self, key: K, value: Value) -> Result<(), BTreeStoreError> {
        let key_buffer_size: u32 = self.static_settings.key_buffer_size.try_into().unwrap();

        let mut tx = self
            .transaction_manager
            .insert_transaction(&self.pages, key_buffer_size);

        self.insert(&mut tx, key, value)?;

        tx.commit::<K>();

        Ok(())
    }

    pub fn insert_one(&self, key: K, value: Value) -> Result<(), BTreeStoreError> {
        self.insert_async(key, value)?;

        self.checkpoint()?;

        Ok(())
    }

    pub fn insert_many(
        &self,
        iter: impl IntoIterator<Item = (K, Value)>,
    ) -> Result<(), BTreeStoreError> {
        let key_buffer_size: u32 = self.static_settings.key_buffer_size.try_into().unwrap();

        let mut tx = self
            .transaction_manager
            .insert_transaction(&self.pages, key_buffer_size);

        for (key, value) in iter {
            self.insert(&mut tx, key, value)?;
        }

        tx.commit::<K>();
        self.checkpoint()?;
        Ok(())
    }

    fn insert<'a>(
        &self,
        tx: &mut InsertTransaction<'a, 'a>,
        key: K,
        value: Value,
    ) -> Result<(), BTreeStoreError> {
        let mut backtrack = tx.backtrack();
        backtrack.search_for(&key);

        let needs_recurse = {
            let leaf = backtrack.get_next()?.unwrap();
            let leaf_id = leaf.id();
            self.insert_in_leaf(leaf, key, value)?
                .map(|(split_key, new_node)| (leaf_id, split_key, new_node))
        };

        if let Some((leaf_id, split_key, new_node)) = needs_recurse {
            let id =
                backtrack.add_new_node(new_node.to_page(), self.static_settings.key_buffer_size)?;

            if backtrack.has_next() {
                self.insert_in_internals(split_key, id, &mut backtrack)?;
            } else {
                let new_root = self.create_internal_node(leaf_id, id, split_key);
                backtrack.new_root(new_root.to_page(), self.static_settings.key_buffer_size)?;
            }
        }

        Ok(())
    }

    pub(crate) fn insert_in_leaf<'a, 'b: 'a>(
        &self,
        leaf: PageRefMut<'a, 'b>,
        key: K,
        value: Value,
    ) -> Result<Option<(K, Node<K, MemPage>)>, BTreeStoreError> {
        let update = {
            let key_size = usize::try_from(self.static_settings.key_buffer_size).unwrap();
            let page_size = usize::try_from(self.static_settings.page_size).unwrap();
            let mut allocate = || {
                let uninit = MemPage::new(page_size);
                Node::<K, MemPage>::new_leaf(key_size, uninit)
            };

            let insert_status = leaf.as_node_mut(key_size, move |mut node: Node<K, &mut [u8]>| {
                node.as_leaf_mut().insert(key, value, &mut allocate)
            });

            match insert_status {
                LeafInsertStatus::Ok => None,
                LeafInsertStatus::DuplicatedKey(_k) => {
                    return Err(crate::BTreeStoreError::DuplicatedKey)
                }
                LeafInsertStatus::Split(split_key, node) => Some((split_key, node)),
            }
        };

        Ok(update)
    }

    // this function recurses on the backtrack splitting internal nodes as needed
    fn insert_in_internals(
        &self,
        key: K,
        to_insert: PageId,
        backtrack: &mut InsertBacktrack<K>,
    ) -> Result<(), BTreeStoreError> {
        let mut split_key = key;
        let mut right_id = to_insert;
        loop {
            let (current_id, new_split_key, new_node) = {
                let node = backtrack.get_next()?.unwrap();
                let node_id = node.id();
                let key_size = usize::try_from(self.static_settings.key_buffer_size).unwrap();
                let page_size = self.static_settings.page_size.try_into().unwrap();
                let mut allocate = || {
                    let uninit = MemPage::new(page_size);
                    Node::new_internal(key_size, uninit)
                };

                match node.as_node_mut(key_size, |mut node| {
                    node.as_internal_mut()
                        .insert(split_key, right_id, &mut allocate)
                }) {
                    InternalInsertStatus::Ok => return Ok(()),
                    InternalInsertStatus::Split(split_key, new_node) => {
                        (node_id, split_key, new_node)
                    }
                    _ => unreachable!(),
                }
            };

            let new_id =
                backtrack.add_new_node(new_node.to_page(), self.static_settings.key_buffer_size)?;

            if backtrack.has_next() {
                // set values to insert in next iteration (recurse on parent)
                split_key = new_split_key;
                right_id = new_id;
            } else {
                let left_id = current_id;
                let right_id = new_id;
                let new_root = self.create_internal_node(left_id, right_id, new_split_key);

                backtrack.new_root(new_root.to_page(), self.static_settings.key_buffer_size)?;
                return Ok(());
            }
        }
    }

    // Used when the current root needs a split
    fn create_internal_node(
        &self,
        left_child: PageId,
        right_child: PageId,
        key: K,
    ) -> Node<K, MemPage> {
        let page = MemPage::new(self.static_settings.page_size.try_into().unwrap());
        let mut node = Node::new_internal(
            self.static_settings.key_buffer_size.try_into().unwrap(),
            page,
        );

        node.as_internal_mut()
            .insert_first(key, left_child, right_child);

        node
    }

    pub fn lookup(&self, key: &K) -> Option<Value> {
        let read_transaction = self.transaction_manager.read_transaction(&self.pages);

        let page_ref = self.search(&read_transaction, key);

        let key_buffer_size = self.static_settings.key_buffer_size.try_into().unwrap();

        page_ref.as_node(key_buffer_size, |node: Node<K, &[u8]>| {
            match node.as_leaf().keys().binary_search(key) {
                Ok(pos) => Some(*node.as_leaf().values().get(pos).borrow()),
                Err(_) => None,
            }
        })
    }

    fn search<'a>(&'a self, tx: &'a ReadTransaction, key: &K) -> PageHandle<'a, borrow::Immutable> {
        let mut current = tx.get_page(tx.root()).unwrap();

        let key_buffer_size = self.static_settings.key_buffer_size.try_into().unwrap();

        loop {
            let new_current = current.as_node(key_buffer_size, |node: Node<K, &[u8]>| {
                node.try_as_internal().map(|inode| {
                    let upper_pivot = match inode.keys().binary_search(key) {
                        Ok(pos) => Some(pos + 1),
                        Err(pos) => Some(pos),
                    }
                    .filter(|pos| pos < &inode.children().len());

                    let new_current_id = if let Some(upper_pivot) = upper_pivot {
                        inode.children().get(upper_pivot)
                    } else {
                        let last = inode.children().len().checked_sub(1).unwrap();
                        inode.children().get(last)
                    };

                    tx.get_page(new_current_id).unwrap()
                })
            });

            if let Some(new_current) = new_current {
                current = new_current;
            } else {
                // found leaf
                break;
            }
        }

        current
    }

    // TODO: the delete function needs a decent cleanup/refactor
    pub fn delete(&self, key: &K) -> Result<(), BTreeStoreError> {
        let key_buffer_size: u32 = self.static_settings.key_buffer_size;
        let mut tx = self
            .transaction_manager
            .insert_transaction(&self.pages, key_buffer_size);

<<<<<<< HEAD
        {
            let mut backtrack = tx.delete_backtrack();

            backtrack.search_for(key);

            // path loaded (cloned) in transaction

            // let (leaf, parent, anchor, left, right) = backtrack.get_next()?.unwrap();
            let DeleteNextElement {
                next,
                parent,
                anchor,
                left,
                right,
            } = backtrack.get_next()?.unwrap();

            let mut leaf = next;
            let leaf_id = leaf.id();

=======
        let backtrack = tx.delete_backtrack();

        backtrack.search_for(key);

        // path loaded (cloned) in transaction

        // let (leaf, parent, anchor, left, right) = backtrack.get_next()?.unwrap();
        let DeleteNextElement {
            next,
            parent,
            anchor,
            left,
            right,
        } = backtrack.get_next()?.unwrap();

        let leaf = next;
        let leaf_id = leaf.page_id;

        let rebalance_result = {
>>>>>>> d11a15a... start adding leaf delete
            let delete_status = leaf.as_node_mut(key_buffer_size as usize, |mut node| {
                node.as_leaf_mut().delete(key)
            })?;

            match delete_status {
                LeafDeleteStatus::Ok => return Ok(()),
                LeafDeleteStatus::NeedsRebalance => (),
            };

            let is_empty = leaf.as_node(key_buffer_size as usize, |root: Node<K, &[u8]>| {
                root.as_leaf().keys().len() == 0
            });

            if let None = parent {
                if is_empty {
                    // do something?
                }
                return Ok(());
            };

            let parent = parent.unwrap();
            let parent_id = parent.page_id;

            let rebalance_result =
                leaf.as_node_mut(key_buffer_size as usize, |mut node: Node<K, &mut [u8]>| {
                    let left_sibling = left.and_then(|id| backtrack.mut_sibling(id));

                    let right_sibling = right.and_then(|id| backtrack.mut_sibling(id));

                    let siblings = SiblingsArg::new_from_options(left_sibling, right_sibling);

                    node.as_leaf_mut()
                        .rebalance(RebalanceArgs {
                            parent,
                            parent_anchor: anchor,
                            siblings,
                        })
                        .expect("couldn't rebalance leaf")
                });

            rebalance_result
        };

        match rebalance_result {
            RebalanceResult::TookKeyFromLeft => {}
            RebalanceResult::TookKeyFromRight => {}
            RebalanceResult::MergeIntoLeft => {
                tx.delete_node(leaf_id);

                self.delete_internal(
                    anchor.expect("merged into left sibling, but anchor is None"),
                    &mut backtrack,
                );
            }
            RebalanceResult::MergeIntoSelf => {
                self.delete_internal(anchor.map_or(0, |a| a + 1), &mut backtrack);
                tx.delete_node(right.unwrap());
            }
        };

        tx.commit::<K>();

        Ok(())
    }

    fn delete_internal(&self, anchor: usize, tx: &mut DeleteBacktrack<K>) {
        unimplemented!()
        //     enum NeedsRebalance {
        //         ShouldRecurse {
        //             rebalance_result: RebalanceResult,
        //             right_id: Option<PageId>,
        //             parent_id: PageId,
        //             parent_anchor: Option<usize>,
        //             self_id: PageId,
        //         },
        //         DeleteRoot {
        //             new_root: PageId,
        //         },
        //     }

        //     let after_delete = {
        //         let (node, parent, parent_anchor, left, right) = tx.get_next::<K>().unwrap();

        //         let delete_status = node.as_node_mut(|mut node: Node<K, &mut [u8]>| {
        //             let mut node = node.as_internal_mut().unwrap();

        //             node.delete_key_children(anchor)
        //         });

        //         match delete_status {
        //             InternalDeleteStatus::Ok => return,
        //             InternalDeleteStatus::NeedsRebalance => (),
        //         };

        //         if let None = parent {
        //             // the root is not rebalanced, but if it is empty then it can
        //             // be deleted
        //             let is_empty = node
        //                 .as_node(|root: Node<K, &[u8]>| root.as_internal().unwrap().keys().len() == 0);

        //             // after deleting a key at position `anchor` and its right children, the left sibling
        //             // is in position == anchor

        //             if is_empty {
        //                 assert!(anchor == 0);
        //                 let new_root = node.as_node(|node: Node<K, &[u8]>| {
        //                     node.as_internal().unwrap().children().get(anchor).unwrap()
        //                 });
        //                 NeedsRebalance::DeleteRoot { new_root }
        //             } else {
        //                 // the root is not rebalanced
        //                 return;
        //             }
        //         } else {
        //             let left_sibling = left.and_then(|id| self.pages.get_page(id));

        //             let right_sibling = right.and_then(|id| self.pages.get_page(id));

        //             let parent = parent.unwrap();
        //             let parent_id = parent.page_id;

        //             let rebalance_result = node
        //                 .as_node_mut(|mut node: Node<K, &mut [u8]>| {
        //                     let siblings = SiblingsArg::new_from_options(
        //                         left_sibling.clone(),
        //                         right_sibling.clone(),
        //                     );

        //                     node.as_internal_mut().unwrap().rebalance(RebalanceArgs {
        //                         parent,
        //                         parent_anchor,
        //                         siblings,
        //                     })
        //                 })
        //                 .expect("couldn't rebalance internal node");

        //             NeedsRebalance::ShouldRecurse {
        //                 rebalance_result,
        //                 right_id: right,
        //                 parent_id,
        //                 parent_anchor,
        //                 self_id: node.page_id,
        //             }
        //         }
        //     };

        //     match after_delete {
        //         NeedsRebalance::ShouldRecurse {
        //             rebalance_result,
        //             right_id,
        //             parent_id,
        //             parent_anchor,
        //             self_id,
        //         } => match rebalance_result {
        //             RebalanceResult::TookKeyFromLeft(mut left_clon) => {
        //                 let old_id = left_clon.page_id;
        //                 left_clon.page_id = tx.new_id();
        //                 tx.add_non_search_path_node::<K>(old_id, left_clon, parent_id);
        //             }
        //             RebalanceResult::TookKeyFromRight(mut right_clon) => {
        //                 let old_id = right_clon.page_id;
        //                 right_clon.page_id = tx.new_id();
        //                 tx.add_non_search_path_node::<K>(old_id, right_clon, parent_id);
        //             }
        //             RebalanceResult::MergeIntoLeft(mut left_clon) => {
        //                 let old_id = left_clon.page_id;
        //                 left_clon.page_id = tx.new_id();
        //                 tx.add_non_search_path_node::<K>(old_id, left_clon, parent_id);

        //                 tx.delete_node(self_id);
        //                 self.delete_internal(
        //                     parent_anchor.expect("merged into left sibling, but anchor is None"),
        //                     tx,
        //                 );
        //             }
        //             RebalanceResult::MergeIntoSelf => {
        //                 let anchor = parent_anchor.clone().map_or(0, |n| n + 1);
        //                 tx.delete_node(right_id.unwrap());
        //                 self.delete_internal(anchor, tx);
        //             }
        //         },
        //         NeedsRebalance::DeleteRoot { new_root } => {
        //             tx.replace_root(new_root);
        //         }
        //     }
    }
}

impl<K> Drop for BTree<K> {
    fn drop(&mut self) {
        let mut guard = self.metadata.lock().unwrap();
        let (metadata, metadata_file) = &mut *guard;

        metadata_file.seek(SeekFrom::Start(0)).unwrap();
        metadata.write(metadata_file).unwrap();

        self.pages
            .read()
            .sync_file()
            .expect("tree file sync failed");
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate tempfile;
    use super::*;
    use crate::tests::U64Key;
    use crate::Key;
    use std::sync::Arc;
    use tempfile::tempfile;

    impl<K> BTree<K>
    where
        K: Key,
    {
        fn key_buffer_size(&self) -> u32 {
            self.static_settings.key_buffer_size
        }

        fn page_size(&self) -> u16 {
            self.static_settings.page_size
        }

        pub fn debug_print(&self) {
            let read_tx = self.transaction_manager.read_transaction(&self.pages);
            let root_id = read_tx.root();

            // TODO: get the next page but IN the read transaction
            for n in 1..self.metadata.lock().unwrap().0.page_manager.next_page() {
                let pages = self.pages.read();
                let page_ref = pages.get_page(n).unwrap();

                println!("-----------------------");
                println!("PageId: {}", n);

                if n == root_id {
                    println!("ROOT");
                }

                let key_size = self.key_buffer_size().try_into().unwrap();

                page_ref.as_node(key_size, |node: Node<K, &[u8]>| match node.get_tag() {
                    node::NodeTag::Internal => {
                        println!("Internal Node");
                        println!("keys: ");
                        for k in node.as_internal().keys().into_iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("children: ");
                        for c in node.as_internal().children().into_iter() {
                            println!("{:?}", c.borrow());
                        }
                    }
                    node::NodeTag::Leaf => {
                        println!("Leaf Node");
                        println!("keys: ");
                        for k in node.as_leaf().keys().into_iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("values: ");
                        for v in node.as_leaf().values().into_iter() {
                            println!("{:?}", v.borrow());
                        }
                    }
                });
                println!("-----------------------");
            }
        }
    }

    fn new_tree() -> BTree<U64Key> {
        let metadata_file = tempfile().unwrap();
        let tree_file = tempfile().unwrap();
        let static_file = tempfile().unwrap();

        let page_size = 88;

        let tree: BTree<U64Key> = BTree::new(
            metadata_file,
            tree_file,
            static_file,
            page_size,
            size_of::<U64Key>().try_into().unwrap(),
        )
        .unwrap();

        tree
    }

    use std::mem::size_of;
    #[test]
    fn insert_many() {
        let tree = new_tree();

        let n: u64 = 2000;

        tree.insert_many((0..n).into_iter().map(|i| (U64Key(i), i)))
            .unwrap();

        tree.debug_print();

        for i in 0..n {
            assert_eq!(tree.lookup(&U64Key(dbg!(i))).expect("Key not found"), i);
        }
    }

    #[quickcheck]
    fn qc_inserted_keys_are_found(xs: Vec<(u64, u64)>) -> bool {
        println!("start qc test");
        let mut reference = std::collections::BTreeMap::new();

        let tree = new_tree();

        // we insert first in the reference in order to get rid of duplicates
        for (xk, xv) in xs {
            reference.entry(xk.clone()).or_insert(xv.clone());
        }

        tree.insert_many(reference.iter().map(|(k, v)| (U64Key(*k), *v)))
            .unwrap();

        let prop = reference
            .iter()
            .all(|(k, v)| match tree.lookup(&U64Key(*dbg!(k))) {
                Some(l) => *v == l,
                None => false,
            });

        prop
    }

    #[test]
    fn saves_and_restores_right() {
        let key_buffer_size: u32 = size_of::<U64Key>().try_into().unwrap();
        let page_size = 86u16;
        {
            let metadata_file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open("metadata")
                .expect("Couldn't create metadata file");

            let tree_file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open("tree")
                .expect("Couldn't create pages file");

            let static_file = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open("static")
                .expect("Couldn't create pages file");

            BTree::<U64Key>::new(
                metadata_file,
                tree_file,
                static_file,
                page_size,
                key_buffer_size,
            )
            .unwrap();
        }

        {
            let restored_tree =
                BTree::<U64Key>::open("metadata", "tree", "static").expect("restore to work");
            assert_eq!(restored_tree.key_buffer_size(), key_buffer_size);
            assert_eq!(restored_tree.page_size(), page_size);
        }

        std::fs::remove_file("tree").unwrap();
        std::fs::remove_file("metadata").unwrap();
        std::fs::remove_file("static").unwrap();
    }

    #[test]
    fn multireads() {
        let tree = new_tree();
        let n: u64 = 2000;

        tree.insert_many((0u64..n).into_iter().map(|i| (U64Key(i), i)))
            .unwrap();

        for i in 0..n {
            assert_eq!(tree.lookup(&U64Key(i)).expect("Key not found"), i);
        }

        use rand::seq::SliceRandom;
        use std::sync::Barrier;
        use std::thread;

        let mut handles = Vec::with_capacity(10);
        let barrier = Arc::new(Barrier::new(10));
        let index = Arc::new(tree);

        for _ in 0..10 {
            let c = barrier.clone();

            let index = index.clone();

            handles.push(thread::spawn(move || {
                let mut queries: Vec<u64> = (0..n).collect();
                let mut rng = rand::thread_rng();

                queries.shuffle(&mut rng);
                c.wait();
                for i in queries {
                    assert_eq!(index.lookup(&U64Key(i)).expect("Key not found"), i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    #[ignore]
    fn multiwrites() {
        let tree = new_tree();

        use rand::seq::SliceRandom;
        use std::sync::{Arc, Barrier};
        use std::thread;

        let mut read_handles = Vec::with_capacity(3);
        let mut write_handles = Vec::with_capacity(3);
        let barrier = Arc::new(Barrier::new(3));
        let index = Arc::new(tree);

        let n = 3000;
        let num_write_threads = 3;
        for thread_num in 0..num_write_threads {
            let c = barrier.clone();
            let index = index.clone();

            write_handles.push(thread::spawn(move || {
                let mut inserts: Vec<u64> = ((n * thread_num)..n * (thread_num + 1)).collect();
                let mut rng = rand::thread_rng();
                inserts.shuffle(&mut rng);
                c.wait();

                for i in inserts {
                    index
                        .insert_async(U64Key(i), i)
                        .expect("duplicated insert in disjoint threads");
                }
            }));
        }

        for thread_num in 0..3 {
            let index = index.clone();

            read_handles.push(thread::spawn(move || {
                // just to make some noise
                while let None = index.lookup(&U64Key(thread_num * n + 500)) {
                    ()
                }
            }));
        }

        for handle in write_handles {
            handle.join().unwrap();
        }

        for handle in read_handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn is_send() {
        // test (at compile time) that certain types implement the auto-trait Send, either directly for
        // pointer-wrapping types or transitively for types with all Send fields

        fn is_send<T: Send>() {
            // dummy function just used for its parameterized type bound
        }

        is_send::<BTree<U64Key>>();
    }
    #[test]
    fn is_sync() {
        // test (at compile time) that certain types implement the auto-trait Sync

        fn is_sync<T: Sync>() {
            // dummy function just used for its parameterized type bound
        }

        is_sync::<BTree<U64Key>>();
    }
}
