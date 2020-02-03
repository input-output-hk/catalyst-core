mod metadata;
mod node;
mod page_manager;
mod pages;
#[allow(dead_code)]
mod version;

use version::*;

use crate::mem_page::MemPage;
use crate::BTreeStoreError;
use metadata::{Metadata, StaticSettings};
use node::{InternalInsertStatus, LeafInsertStatus, Node};
use pages::*;
use std::borrow::Borrow;

use crate::{Key, Value};

use std::convert::{TryFrom, TryInto};
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom};
use std::marker::PhantomData;
use std::path::Path;
use std::sync::Mutex;

pub(crate) type PageId = u32;

pub struct BTree<K> {
    // The metadata file contains the latests confirmed version of the tree
    // this is, the root node, and the list of free pages
    metadata: Mutex<(Metadata, File)>,
    static_settings: StaticSettings,
    pages: Pages,
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
        let mut root_page = MemPage::new(page_size.try_into().unwrap());
        Node::<K, &mut [u8]>::new_leaf(key_buffer_size.try_into().unwrap(), root_page.as_mut());

        let mut metadata = Metadata::new();

        let pages_storage = crate::storage::MmapStorage::new(tree_file)?;

        let pages = Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size: page_size.try_into().unwrap(),
            key_buffer_size,
        });

        let first_page_id = metadata.page_manager.new_id();

        pages
            .write_page(Page {
                page_id: first_page_id,
                key_buffer_size,
                mem_page: root_page,
            })
            .expect("Couldn't write first page");

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

        let pages = Pages::new(PagesInitializationParams {
            storage: pages_storage,
            page_size: static_settings.page_size,
            key_buffer_size: static_settings.key_buffer_size,
        });

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

    pub fn insert_async(&self, key: K, value: Value) -> Result<(), BTreeStoreError> {
        let mut tx = self.transaction_manager.insert_transaction(&self.pages);
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
        let mut tx = self.transaction_manager.insert_transaction(&self.pages);

        for (key, value) in iter {
            self.insert(&mut tx, key, value)?;
        }

        tx.commit::<K>();
        self.checkpoint()?;
        Ok(())
    }

    fn insert<'a>(
        &self,
        tx: &mut InsertTransactionBuilder<'a, 'a>,
        key: K,
        value: Value,
    ) -> Result<(), BTreeStoreError> {
        let mut backtrack = tx.backtrack();
        backtrack.search_for(&key);

        let needs_recurse = {
            let leaf = backtrack.get_next().unwrap();
            self.insert_in_leaf(leaf, key, value)?
                .map(|(split_key, new_node)| (leaf.id(), split_key, new_node))
        };

        if let Some((leaf_id, split_key, new_node)) = needs_recurse {
            let id =
                backtrack.add_new_node(new_node.to_page(), self.static_settings.key_buffer_size);

            if backtrack.has_next() {
                self.insert_in_internals(split_key, id, &mut backtrack)?;
            } else {
                let new_root = self.create_internal_node(leaf_id, id, split_key);
                backtrack.new_root(new_root.to_page(), self.static_settings.key_buffer_size);
            }
        }

        Ok(())
    }

    pub(crate) fn insert_in_leaf<'a>(
        &self,
        leaf: &mut Page,
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

            let insert_status = leaf.as_node_mut(move |mut node: Node<K, &mut [u8]>| {
                node.as_leaf_mut()
                    .unwrap()
                    .insert(key, value, &mut allocate)
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
                let node = backtrack.get_next().unwrap();
                let node_id = node.id();
                let key_size = usize::try_from(self.static_settings.key_buffer_size).unwrap();
                let mut allocate = || {
                    let uninit = MemPage::new(self.static_settings.page_size.try_into().unwrap());
                    Node::new_internal(key_size, uninit)
                };

                match node.as_node_mut(|mut node| {
                    node.as_internal_mut()
                        .unwrap()
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
                backtrack.add_new_node(new_node.to_page(), self.static_settings.key_buffer_size);

            if backtrack.has_next() {
                // set values to insert in next iteration (recurse on parent)
                split_key = new_split_key;
                right_id = new_id;
            } else {
                let left_id = current_id;
                let right_id = new_id;
                let new_root = self.create_internal_node(left_id, right_id, new_split_key);

                backtrack.new_root(new_root.to_page(), self.static_settings.key_buffer_size);
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
            .unwrap()
            .insert_first(key, left_child, right_child);

        node
    }

    pub fn lookup(&self, key: &K) -> Option<Value> {
        let page_ref = self.search(key);

        page_ref.as_node(|node: Node<K, &[u8]>| {
            match node.as_leaf().unwrap().keys().binary_search(key) {
                Ok(pos) => node
                    .as_leaf()
                    .unwrap()
                    .values()
                    .get(pos)
                    .map(|n| *n.borrow()),
                Err(_) => None,
            }
        })
    }

    fn search(&self, key: &K) -> PageRef {
        // TODO: Care, requesting a read transaction should enforce that it's not released until is finished
        // in this case, this acts like a lock, but if it does get released then the data may be overwritten
        let read_transaction = self.transaction_manager.read_transaction();

        let mut current = self.pages.get_page(read_transaction.root()).unwrap();

        loop {
            let new_current: Option<PageRef> = current.as_node(|node: Node<K, &[u8]>| {
                node.as_internal().map(|inode| {
                    let upper_pivot = match inode.keys().binary_search(key) {
                        Ok(pos) => Some(pos + 1),
                        Err(pos) => Some(pos),
                    }
                    .filter(|pos| pos < &inode.children().len());

                    let new_current_id = if let Some(upper_pivot) = upper_pivot {
                        inode.children().get(upper_pivot).unwrap().clone()
                    } else {
                        let last = inode.children().len().checked_sub(1).unwrap();
                        inode.children().get(last).unwrap().clone()
                    };

                    self.pages.get_page(new_current_id).unwrap()
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
}

impl<K> Drop for BTree<K> {
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
    use super::*;
    use crate::tests::U64Key;
    use crate::Key;
    use std::sync::Arc;

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
            let read_tx = self.transaction_manager.read_transaction();
            let root_id = read_tx.root();

            // TODO: get the next page but IN the read transaction
            for n in 1..self.metadata.lock().unwrap().0.page_manager.next_page() {
                let page_ref = self.pages.get_page(n).unwrap();

                println!("-----------------------");
                println!("PageId: {}", n);

                if n == root_id {
                    println!("ROOT");
                }
                page_ref.as_node(|node: Node<K, &[u8]>| match node.get_tag() {
                    node::NodeTag::Internal => {
                        println!("Internal Node");
                        println!("keys: ");
                        for k in node.as_internal().unwrap().keys().into_iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("children: ");
                        for c in node.as_internal().unwrap().children().into_iter() {
                            println!("{:?}", c.borrow());
                        }
                    }
                    node::NodeTag::Leaf => {
                        println!("Leaf Node");
                        println!("keys: ");
                        for k in node.as_leaf().unwrap().keys().into_iter() {
                            println!("{:?}", k.borrow());
                        }
                        println!("values: ");
                        for v in node.as_leaf().unwrap().values().into_iter() {
                            println!("{:?}", v.borrow());
                        }
                    }
                });
                println!("-----------------------");
            }
        }
    }

    struct RAIITree(BTree<U64Key>, Vec<String>);

    use std::ops::Deref;
    impl Deref for RAIITree {
        type Target = BTree<U64Key>;
        fn deref(&self) -> &BTree<U64Key> {
            &self.0
        }
    }

    impl Drop for RAIITree {
        fn drop(&mut self) {
            for file_name in self.1.iter() {
                std::fs::remove_file(file_name).unwrap();
            }
        }
    }

    use crate::btreeindex::tests::rand::Rng as _;
    fn new_tree() -> RAIITree {
        use rand::distributions::Alphanumeric;
        let file_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        let metadata_file_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        let static_settings_file_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>();

        let tree_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(file_name.clone())
            .expect("Couldn't create pages file");

        let metadata_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(metadata_file_name.clone())
            .expect("Couldn't create metadata file");

        let static_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(static_settings_file_name.clone())
            .expect("Couldn't create metadata file");

        let tree: BTree<U64Key> = BTree::new(
            metadata_file,
            tree_file,
            static_file,
            86,
            size_of::<U64Key>().try_into().unwrap(),
        )
        .unwrap();

        RAIITree(
            tree,
            vec![file_name, metadata_file_name, static_settings_file_name],
        )
    }

    use std::mem::size_of;
    #[test]
    fn insert_many() {
        let tree = new_tree();

        let n: u64 = 2000;

        tree.insert_many((0..n).into_iter().map(|i| (U64Key(i), i)))
            .unwrap();

        for i in 0..n {
            assert_eq!(tree.lookup(&U64Key(i)).expect("Key not found"), i);
        }
    }

    #[quickcheck]
    #[ignore]
    fn qc_inserted_keys_are_found(xs: Vec<(u64, u64)>) -> bool {
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
            .all(|(k, v)| match tree.lookup(&U64Key(*k)) {
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

        let restored_tree =
            BTree::<U64Key>::open("metadata", "tree", "static").expect("restore to work");
        assert_eq!(restored_tree.key_buffer_size(), key_buffer_size);
        assert_eq!(restored_tree.page_size(), page_size);

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
