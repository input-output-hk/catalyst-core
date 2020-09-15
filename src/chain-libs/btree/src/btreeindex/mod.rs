mod backtrack;
mod metadata;
// FIXME: allow dead code momentarily, because all of the delete algorithms are unused, and placing the directive with more granularity would be too troublesome
mod iter;
mod node;
mod page_manager;
mod pages;
mod version_management;

use version_management::transaction::{PageRef, PageRefMut, ReadTransaction, WriteTransaction};
use version_management::*;

use crate::mem_page::MemPage;
use crate::BTreeStoreError;
use metadata::{Metadata, StaticSettings};
use node::internal_node::InternalDeleteStatus;
use node::leaf_node::LeafDeleteStatus;
use node::{
    InternalInsertStatus, LeafInsertStatus, Node, NodeRef, NodeRefMut, RebalanceResult, SiblingsArg,
};
use pages::{Pages, PagesInitializationParams};
use std::borrow::Borrow;

use crate::FixedSize;

use backtrack::{DeleteBacktrack, InsertBacktrack, UpdateBacktrack};
use std::convert::{TryFrom, TryInto};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::marker::PhantomData;
use std::ops::RangeBounds;
use std::path::Path;
use std::sync::Mutex;

use iter::BTreeIterator;

pub type PageId = u32;
const NODES_PER_PAGE: u64 = 2000;

pub struct BTree<K, V> {
    // The metadata file contains the latests confirmed version of the tree
    // this is, the root node, and the list of free pages
    metadata: Mutex<(Metadata, File)>,
    static_settings: StaticSettings,
    pages: Pages,
    transaction_manager: TransactionManager,
    phantom_keys: PhantomData<[K]>,
    phantom_values: PhantomData<[V]>,
}

/// Views over continous arrays of data. The buffer represents the total capacity
/// but they keep track of the current actual length of items
use crate::arrayview::ArrayView;
pub(crate) type Children<'a> = ArrayView<'a, &'a [u8], PageId>;
pub(crate) type ChildrenMut<'a> = ArrayView<'a, &'a mut [u8], PageId>;
pub(crate) type Values<'a, V> = ArrayView<'a, &'a [u8], V>;
pub(crate) type ValuesMut<'a, V> = ArrayView<'a, &'a mut [u8], V>;
pub(crate) type Keys<'a, K> = ArrayView<'a, &'a [u8], K>;
pub(crate) type KeysMut<'a, K> = ArrayView<'a, &'a mut [u8], K>;

type SplitKeyNodePair<K> = (K, Node<K, MemPage>);

impl<'me, K: 'me, V> BTree<K, V>
where
    K: FixedSize,
    V: FixedSize,
{
    // TODO: add a builder with defaults?
    pub fn new(
        metadata_file: File,
        tree_file: File,
        mut static_settings_file: File,
        page_size: u16,
        key_buffer_size: u32,
    ) -> Result<BTree<K, V>, BTreeStoreError> {
        let mut metadata = Metadata::new();

        let pages_storage =
            crate::storage::MmapStorage::new(tree_file, page_size as u64 * NODES_PER_PAGE)?;

        let pages = Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size,
        });

        let first_page_id = metadata.page_manager.new_id();

        let mut root_page = pages.mut_page(first_page_id)?;

        root_page.as_slice(|page| {
            Node::<K, &mut [u8]>::new_leaf::<V>(page);
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
            pages,
            static_settings,
            transaction_manager,
            phantom_keys: PhantomData,
            phantom_values: PhantomData,
        })
    }

    pub fn open(
        metadata_file: impl AsRef<Path>,
        tree_file: impl AsRef<Path>,
        static_settings_file: impl AsRef<Path>,
    ) -> Result<BTree<K, V>, BTreeStoreError> {
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

        let tree_file = OpenOptions::new().write(true).read(true).open(tree_file)?;
        let pages_storage = crate::storage::MmapStorage::new(
            tree_file,
            static_settings.page_size as u64 * NODES_PER_PAGE,
        )?;

        let pages = Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size: static_settings.page_size,
        });

        let transaction_manager = TransactionManager::new(&metadata);

        Ok(BTree {
            metadata: Mutex::new((metadata, metadata_file)),
            pages,
            static_settings,
            transaction_manager,
            phantom_keys: PhantomData,
            phantom_values: PhantomData,
        })
    }

    // sync files to disk and collect old transactions pages
    pub(crate) fn checkpoint(&self) -> Result<(), BTreeStoreError> {
        if let Some(checkpoint) = self.transaction_manager.collect_pending() {
            let new_metadata = checkpoint.new_metadata;

            self.pages.sync_file()?;

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

    pub fn insert_async(&self, key: K, value: V) -> Result<(), BTreeStoreError> {
        let mut tx = self.transaction_manager.write_transaction(&self.pages);

        self.insert(&mut tx, key, value)?;

        tx.commit::<K>();

        Ok(())
    }

    pub fn insert_one(&self, key: K, value: V) -> Result<(), BTreeStoreError> {
        self.insert_async(key, value)?;

        self.checkpoint()?;

        Ok(())
    }

    pub fn insert_many(
        &self,
        iter: impl IntoIterator<Item = (K, V)>,
    ) -> Result<(), BTreeStoreError> {
        let mut tx = self.transaction_manager.write_transaction(&self.pages);

        for (key, value) in iter {
            self.insert(&mut tx, key, value)?;
        }

        tx.commit::<K>();
        self.checkpoint()?;
        Ok(())
    }

    fn insert<'a>(
        &self,
        tx: &mut WriteTransaction<'a, 'a>,
        key: K,
        value: V,
    ) -> Result<(), BTreeStoreError> {
        let mut backtrack = InsertBacktrack::new_search_for(tx, &key);

        let needs_recurse = {
            let leaf = backtrack.get_next()?.unwrap();
            let leaf_id = leaf.id();
            self.insert_in_leaf(leaf, key, value)?
                .map(|(split_key, new_node)| (leaf_id, split_key, new_node))
        };

        if let Some((leaf_id, split_key, new_node)) = needs_recurse {
            let id = backtrack.add_new_node(new_node.into_page())?;

            if backtrack.has_next() {
                self.insert_in_internals(split_key, id, &mut backtrack)?;
            } else {
                let new_root = self.create_internal_node(leaf_id, id, split_key);
                backtrack.new_root(new_root.into_page())?;
            }
        }

        Ok(())
    }

    pub(crate) fn insert_in_leaf<'a>(
        &self,
        mut leaf: PageRefMut<'a>,
        key: K,
        value: V,
    ) -> Result<Option<SplitKeyNodePair<K>>, BTreeStoreError> {
        let update = {
            let page_size = usize::try_from(self.static_settings.page_size).unwrap();
            let mut allocate = || {
                let uninit = MemPage::new(page_size);
                Node::<K, MemPage>::new_leaf::<V>(uninit)
            };

            let insert_status = leaf.as_node_mut(move |mut node: Node<K, &mut [u8]>| {
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
                let mut node = backtrack.get_next()?.unwrap();
                let node_id = node.id();
                let page_size = self.static_settings.page_size.try_into().unwrap();
                let mut allocate = || {
                    let uninit = MemPage::new(page_size);
                    Node::new_internal(uninit)
                };

                match node.as_node_mut(|mut node| {
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

            let new_id = backtrack.add_new_node(new_node.into_page())?;

            if backtrack.has_next() {
                // set values to insert in next iteration (recurse on parent)
                split_key = new_split_key;
                right_id = new_id;
            } else {
                let left_id = current_id;
                let right_id = new_id;
                let new_root = self.create_internal_node(left_id, right_id, new_split_key);

                backtrack.new_root(new_root.into_page())?;
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
        let mut node = Node::new_internal(page);

        node.as_internal_mut()
            .insert_first(key, left_child, right_child);

        node
    }

    // we use a function for the return value in order to avoid cloning the value, returning a direct reference is not possible because we need
    // the ReadTransaction to exist in order to keep the page from being reused.
    pub fn get<Q, F, R>(&self, key: &Q, f: F) -> R
    where
        Q: Ord,
        K: Borrow<Q>,
        F: FnOnce(Option<&V>) -> R,
    {
        let read_transaction = self.transaction_manager.read_transaction(&self.pages);

        let page_ref = self.search(&read_transaction, key);

        page_ref.as_node(|node: Node<K, &[u8]>| {
            match node.as_leaf::<V>().keys().binary_search::<Q>(key) {
                // TODO: Find if it is possible to avoid this clone (although it's only important if V is a big type, which should be avoided anyway)
                Ok(pos) => f(Some(node.as_leaf::<V>().values().get(pos).borrow())),
                Err(_) => f(None),
            }
        })
    }

    /// perform a range query. The returned iterator holds a read-only transaction for it's entire lifetime.
    /// This avoids pages to be collected, so it may better for it to not be long-lived.
    pub fn range<R, Q>(&self, range: R) -> BTreeIterator<R, Q, K, V>
    where
        K: Borrow<Q>,
        R: RangeBounds<Q>,
        Q: Ord,
    {
        let read_transaction = self.transaction_manager.read_transaction(&self.pages);

        BTreeIterator::new(read_transaction, range)
    }

    fn search<'a, Q>(&'a self, tx: &'a ReadTransaction, key: &Q) -> PageRef<'a>
    where
        Q: Ord,
        K: Borrow<Q>,
    {
        let mut current = tx.get_page(tx.root()).unwrap();

        loop {
            let new_current = current.as_node(|node: Node<K, &[u8]>| {
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

    pub fn update(&self, key: &K, value: V) -> Result<(), BTreeStoreError> {
        let mut tx = self.transaction_manager.write_transaction(&self.pages);

        UpdateBacktrack::new_search_for(&mut tx, key).update(value)?;

        tx.commit::<K>();
        Ok(())
    }

    /// delete given key from the tree, this doesn't sync the file to disk
    pub fn delete(&self, key: &K) -> Result<(), BTreeStoreError> {
        let mut tx = self.transaction_manager.write_transaction(&self.pages);

        let result = self.delete_async(key, &mut tx);

        tx.commit::<K>();

        result
    }

    fn delete_async<'a>(
        &'a self,
        key: &K,
        tx: &mut WriteTransaction<'a, 'a>,
    ) -> Result<(), BTreeStoreError> {
        let mut backtrack = DeleteBacktrack::new_search_for(tx, key);

        // we can unwrap safely because there is always a leaf in the path
        // delete will return Ok if the key is not in the given leaf
        use backtrack::DeleteNextElement;
        let DeleteNextElement {
            mut next_element,
            mut_context,
        } = backtrack.get_next()?.unwrap();

        let delete_result = next_element
            .next
            .as_node_mut(|mut node| node.as_leaf_mut::<V>().delete(key))?;

        match delete_result {
            LeafDeleteStatus::Ok => return Ok(()),
            LeafDeleteStatus::NeedsRebalance => (),
        };

        // this allows us to get mutable references to out parent and siblings, we only need those when we need to rebalance
        let mut mut_context = match mut_context {
            Some(mut_context) => mut_context,
            // this means we are processing the root node, it is not possible to do any rebalancing because we don't have siblings
            // I think we don't need to do anything here, in theory, we could change the tree height to 0, but we are not tracking the height
            None => return Ok(()),
        };

        let next = &mut next_element.next;
        let left = next_element.left.as_ref();
        let right = next_element.right.as_ref();
        // we need this to know which child we are (what position does this node have in the parent)
        let anchor = next_element.anchor;

        let should_recurse_on_parent: Option<usize> = next.as_node_mut(
            |mut node: Node<K, &mut [u8]>| -> Result<Option<usize>, BTreeStoreError> {
                let siblings = SiblingsArg::new_from_options(left, right);

                match node.as_leaf_mut::<V>().rebalance(siblings)? {
                    RebalanceResult::TakeFromLeft(add_sibling) => {
                        let (sibling, parent) = mut_context.mut_left_sibling();
                        add_sibling.take_key_from_left(parent, anchor, sibling);
                        Ok(None)
                    }
                    RebalanceResult::TakeFromRight(add_sibling) => {
                        let (sibling, parent) = mut_context.mut_right_sibling();
                        add_sibling.take_key_from_right(parent, anchor, sibling);
                        Ok(None)
                    }
                    RebalanceResult::MergeIntoLeft(add_sibling) => {
                        let (sibling, _) = mut_context.mut_left_sibling();
                        add_sibling.merge_into_left(sibling);
                        mut_context.delete_node();
                        // the anchor is the the index of the key that splits the left sibling and the node, it's only None if the current node
                        // it's the leftmost (and thus has no left sibling)
                        Ok(Some(
                            anchor.expect("merged into left sibling, but anchor is None"),
                        ))
                    }
                    RebalanceResult::MergeIntoSelf(add_sibling) => {
                        let (sibling, _) = mut_context.mut_right_sibling();
                        add_sibling.merge_into_self(sibling);
                        mut_context
                            .delete_right_sibling()
                            .expect("can't mutate right sibling");
                        Ok(Some(anchor.map_or(0, |a| a + 1)))
                    }
                }
            },
        )?;

        // we need to do this because `mut_context` has a mutable borrow of the parent, which is the next node to process
        // I don't think adding an additional scope and indentation level is worth it in that case. Geting rid of the closure above may be a better solution
        drop(mut_context);

        if let Some(anchor) = should_recurse_on_parent {
            self.delete_internal(anchor, &mut backtrack)?;
        }

        Ok(())
    }

    fn delete_internal(
        &self,
        anchor: usize,
        tx: &mut DeleteBacktrack<K>,
    ) -> Result<(), BTreeStoreError> {
        let mut anchor_to_delete = anchor;
        while let Some(next_element) = tx.get_next()? {
            let backtrack::DeleteNextElement {
                mut next_element,
                mut_context,
            } = next_element;

            match next_element
                .next
                .as_node_mut(|mut node: Node<K, &mut [u8]>| {
                    let mut node = node.as_internal_mut();
                    node.delete_key_children(anchor_to_delete)
                }) {
                InternalDeleteStatus::Ok => return Ok(()),
                InternalDeleteStatus::NeedsRebalance => (),
            };

            match mut_context {
                None => {
                    // here we are dealing with the root
                    // the root is not rebalanced, but if it is empty then it can
                    // be deleted, and unlike the leaf case, we need to promote it's only remainining child as the new root
                    let is_empty = next_element
                        .next
                        .as_node(|root: Node<K, &[u8]>| root.as_internal().keys().len() == 0);

                    // after deleting a key at position `anchor` and its right children, the left sibling
                    // is in position == anchor

                    if is_empty {
                        debug_assert!(anchor == 0);
                        let new_root = next_element.next.as_node(|node: Node<K, &[u8]>| {
                            node.as_internal().children().get(anchor)
                        });

                        next_element.set_root(new_root);
                    }
                }
                Some(mut mut_context) => {
                    // non-root node
                    // let parent = next_element.parent.unwrap();
                    let anchor = next_element.anchor;
                    let left = next_element.left;
                    let right = next_element.right;

                    // as in the leaf case, the value in the Option is the 'anchor' (pointer) to the deleted node
                    let recurse_on_parent: Option<usize> = next_element.next.as_node_mut(
                        |mut node: Node<K, &mut [u8]>| -> Result<Option<usize>, BTreeStoreError> {
                            let siblings = SiblingsArg::new_from_options(left, right);

                            match node.as_internal_mut().rebalance(siblings)? {
                                RebalanceResult::TakeFromLeft(add_params) => {
                                    let (sibling, parent) = mut_context.mut_left_sibling();
                                    add_params.take_key_from_left(
                                        parent,
                                        anchor.expect(
                                            "left sibling seems to exist but anchor is none",
                                        ),
                                        sibling,
                                    );
                                    Ok(None)
                                }
                                RebalanceResult::TakeFromRight(add_params) => {
                                    let (sibling, parent) = mut_context.mut_right_sibling();
                                    add_params.take_key_from_right(parent, anchor, sibling);
                                    Ok(None)
                                }
                                RebalanceResult::MergeIntoLeft(add_params) => {
                                    let (sibling, parent) = mut_context.mut_left_sibling();
                                    add_params.merge_into_left(parent, anchor, sibling)?;
                                    mut_context.delete_node();
                                    Ok(Some(
                                        anchor
                                            .clone()
                                            .expect("merged into left sibling, but anchor is none"),
                                    ))
                                }
                                RebalanceResult::MergeIntoSelf(add_params) => {
                                    let (sibling, parent) = mut_context.mut_right_sibling();
                                    add_params.merge_into_self(parent, anchor, sibling)?;
                                    let new_anchor = anchor.map_or(0, |n| n + 1);
                                    mut_context
                                        .delete_right_sibling()
                                        .expect("right sibling doesn't exist");
                                    Ok(Some(new_anchor))
                                }
                            }
                        },
                    )?;

                    // (there is no recursive call, we just go the next loop iteration)
                    if let Some(anchor) = recurse_on_parent {
                        anchor_to_delete = anchor;
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}

impl<K, V> Drop for BTree<K, V> {
    fn drop(&mut self) {
        let mut guard = self.metadata.lock().unwrap();
        let (metadata, metadata_file) = &mut *guard;

        metadata_file.seek(SeekFrom::Start(0)).unwrap();
        metadata.write(metadata_file).unwrap();

        self.pages.sync_file().expect("tree file sync failed");
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate tempfile;
    use super::*;
    use crate::tests::U64Key;
    use crate::FixedSize;
    use std::sync::Arc;
    use tempfile::tempfile;

    impl<K> BTree<K, u64>
    where
        K: FixedSize,
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
                let pages = &self.pages;
                let page_ref = pages.get_page(n).unwrap();

                println!("-----------------------");
                println!("PageId: {}", n);

                if n == root_id {
                    println!("ROOT");
                }

                page_ref.as_node(|node: Node<K, &[u8]>| match node.get_tag() {
                    node::NodeTag::Internal => {
                        println!("Internal Node");
                        println!("keys: ");
                        for k in node.as_internal().keys().iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("children: ");
                        for c in node.as_internal().children().iter() {
                            println!("{:?}", c.borrow());
                        }
                    }
                    node::NodeTag::Leaf => {
                        println!("Leaf Node");
                        println!("keys: ");
                        for k in node.as_leaf::<u64>().keys().iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("values: ");
                        for v in node.as_leaf::<u64>().values().iter() {
                            println!("{:?}", v.borrow());
                        }
                    }
                });
                println!("-----------------------");
            }
        }
    }

    pub fn new_tree() -> BTree<U64Key, u64> {
        let metadata_file = tempfile().unwrap();
        let tree_file = tempfile().unwrap();
        let static_file = tempfile().unwrap();

        let page_size = 88;

        let tree: BTree<U64Key, u64> = BTree::new(
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

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        tree.debug_print();

        for i in 0..n {
            assert_eq!(
                tree.get(&U64Key(i), |key| key.cloned())
                    .expect("Key not found"),
                i
            );
        }
    }

    #[quickcheck]
    fn qc_inserted_keys_are_found(xs: Vec<(u64, u64)>) -> bool {
        println!("start qc test");
        let mut reference = std::collections::BTreeMap::new();

        let tree = new_tree();

        // we insert first in the reference in order to get rid of duplicates
        for (xk, xv) in xs {
            reference.entry(xk).or_insert(xv);
        }

        tree.insert_many(reference.iter().map(|(k, v)| (U64Key(*k), *v)))
            .unwrap();

        reference
            .iter()
            .all(|(k, v)| match tree.get(&U64Key(*k), |v| v.cloned()) {
                Some(l) => *v == l,
                None => false,
            })
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

            BTree::<U64Key, u64>::new(
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
                BTree::<U64Key, u64>::open("metadata", "tree", "static").expect("restore to work");
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

        tree.insert_many((0u64..n).map(|i| (U64Key(i), i))).unwrap();

        for i in 0..n {
            assert_eq!(
                tree.get(&U64Key(i), |value| value.cloned())
                    .expect("Key not found"),
                i
            );
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
                    assert_eq!(
                        index
                            .get(&U64Key(i), |v| v.cloned())
                            .expect("Key not found"),
                        i
                    );
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
                while index
                    .get(&U64Key(thread_num * n + 500), |v| v.cloned())
                    .is_none()
                {}
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
    fn can_delete_key() {
        let tree = new_tree();
        let n: u64 = 2000;
        let delete: u64 = 50;

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        let key_to_delete = U64Key(delete);
        assert!(tree.get(&key_to_delete, |v| v.cloned()).is_some());

        tree.delete(&key_to_delete).unwrap();

        tree.debug_print();

        assert!(tree.get(&key_to_delete, |v| v.cloned()).is_none());

        for i in (0..n).filter(|n| *n != delete) {
            assert!(tree.get(&U64Key(i), |v| v.cloned()).is_some());
        }
    }

    #[quickcheck]
    #[ignore]
    fn qc_arbitrary_deletes(xs: Vec<u64>) -> bool {
        let mut reference = std::collections::BTreeMap::new();

        let tree = new_tree();
        let n: u64 = 2000;
        for i in 0..n {
            reference.entry(U64Key(i)).or_insert(i);
        }

        tree.insert_many(reference.iter().map(|(k, v)| (k.clone(), *v)))
            .unwrap();

        for k in xs {
            reference.remove(&U64Key(k));
            tree.delete(&U64Key(k)).unwrap_or(());
            assert!(tree.get(&U64Key(k), |v| v.cloned()).is_none());
        }

        reference
            .iter()
            .all(|(k, v)| match tree.get(k, |v| v.cloned()) {
                Some(l) => *v == l,
                None => false,
            })
    }

    #[test]
    fn test_update() {
        let tree = new_tree();

        let n: u64 = 2000;

        tree.insert_many((0..n).map(|i| (U64Key(i), i))).unwrap();

        assert_eq!(tree.get(&U64Key(100), |v| v.cloned()), Some(100));

        tree.update(&U64Key(100), 120).unwrap();

        assert_eq!(tree.get(&U64Key(100), |v| v.cloned()), Some(120));
    }

    #[test]
    fn is_send() {
        // test (at compile time) that certain types implement the auto-trait Send, either directly for
        // pointer-wrapping types or transitively for types with all Send fields

        fn is_send<T: Send>() {
            // dummy function just used for its parameterized type bound
        }

        is_send::<BTree<U64Key, u64>>();
    }
    #[test]
    fn is_sync() {
        // test (at compile time) that certain types implement the auto-trait Sync

        fn is_sync<T: Sync>() {
            // dummy function just used for its parameterized type bound
        }

        is_sync::<BTree<U64Key, u64>>();
    }
}
