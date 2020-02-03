use super::metadata::Metadata;
use super::page_manager::PageManager;

use super::pages::*;
use super::Node;
use super::PageId;
use crate::mem_page::MemPage;
use crate::Key;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

use std::sync::{Arc, Mutex, MutexGuard, RwLock};

pub(crate) struct TransactionManager {
    latest_version: Arc<RwLock<Arc<Version>>>,
    versions: Mutex<VecDeque<Arc<Version>>>,
    page_manager: Mutex<PageManager>,
}

pub(crate) struct Version {
    root: PageId,
    transaction: WriteTransaction,
}

/// delta-like structure, it has the list of pages that can be collected after no readers are using them
pub(crate) struct WriteTransaction {
    new_root: PageId,
    shadowed_pages: Vec<PageId>,
    next_page_id: PageId,
}

/// this has locks, so no new transaction can occur while this is synced to disk
pub(crate) struct Checkpoint<'a> {
    pub(crate) new_metadata: Metadata,
    page_manager: MutexGuard<'a, PageManager>,
    versions: MutexGuard<'a, VecDeque<Arc<Version>>>,
}

impl Version {
    pub(crate) fn root(&self) -> PageId {
        self.root
    }
}

pub(crate) enum WriteTransactionBuilder<'a, 'index> {
    Insert(InsertTransactionBuilder<'a, 'index>),
}

/// staging area for batched insertions, it will keep track of pages already shadowed and reuse them,
/// it can be used to create a new `Version` at the end with all the insertions done atomically
pub(crate) struct InsertTransactionBuilder<'index, 'locks: 'index> {
    pages: &'index Pages,
    current_root: PageId,
    extra: HashMap<PageId, Page>,
    old_ids: Vec<PageId>,
    current: Option<usize>,
    page_manager: MutexGuard<'locks, PageManager>,
    versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
    current_version: Arc<RwLock<Arc<Version>>>,
}

/// this is basically a stack, but it will rename pointers and interact with the builder in order to reuse
/// already cloned pages
pub(crate) struct InsertBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
where
    K: Key,
{
    builder: &'txbuilder mut InsertTransactionBuilder<'txmanager, 'index>,
    backtrack: Vec<(Option<PageId>, Page)>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
}

impl TransactionManager {
    pub fn new(metadata: &Metadata) -> TransactionManager {
        let latest_version = Arc::new(RwLock::new(Arc::new(Version {
            root: metadata.root,
            transaction: WriteTransaction {
                new_root: metadata.root,
                shadowed_pages: vec![],
                next_page_id: metadata.page_manager.next_page(),
            },
        })));

        let versions = Mutex::new(VecDeque::new());
        let page_manager = Mutex::new(metadata.page_manager.clone());

        TransactionManager {
            latest_version,
            versions,
            page_manager,
        }
    }

    pub fn latest_version(&self) -> Arc<Version> {
        self.latest_version.read().unwrap().clone()
    }

    pub fn read_transaction(&self) -> Arc<Version> {
        self.latest_version()
    }

    pub fn insert_transaction<'me, 'index: 'me>(
        &'me self,
        pages: &'index Pages,
    ) -> InsertTransactionBuilder<'me, 'me> {
        let page_manager = self.page_manager.lock().unwrap();
        let versions = self.versions.lock().unwrap();

        InsertTransactionBuilder {
            current_root: self.latest_version().root(),
            extra: HashMap::new(),
            old_ids: vec![],
            pages,
            current: None,
            page_manager,
            versions,
            current_version: self.latest_version.clone(),
        }
    }

    /// collect versions without readers, in order to reuse its pages (the ones that are shadow in transactions after that)
    pub fn collect_pending(&self) -> Option<Checkpoint> {
        let mut page_manager = self.page_manager.lock().unwrap();
        let mut versions = self.versions.lock().unwrap();

        let mut pages_to_release = vec![];
        let mut next_page_at_end = None;
        let mut new_root = None;

        while versions.len() > 0 && Arc::strong_count(versions.front().unwrap()) == 1 {
            // there is no race conditions between the check and this, because versions is locked and count == 1 means is the only reference
            let version = versions.pop_front().unwrap();
            // FIXME: remove this loop?
            for id in version.transaction.shadowed_pages.iter().cloned() {
                pages_to_release.push(id)
            }

            next_page_at_end = Some(version.transaction.next_page_id);
            new_root = Some(version.transaction.new_root);
        }

        let next_page: PageId = if let Some(next_page) = next_page_at_end {
            next_page
        } else {
            return None;
        };

        for page in pages_to_release {
            page_manager.remove_page(page);
        }

        let page_manager_to_commit = PageManager {
            next_page,
            ..page_manager.clone()
        };

        Some(Checkpoint {
            new_metadata: Metadata {
                root: new_root.unwrap(),
                page_manager: page_manager_to_commit,
            },
            page_manager,
            versions,
        })
    }
}

impl<'txmanager, 'index: 'txmanager> InsertTransactionBuilder<'txmanager, 'index> {
    /// create a staging area for a single insert
    pub(crate) fn backtrack<'me, K>(&'me mut self) -> InsertBacktrack<'me, 'txmanager, 'index, K>
    where
        K: Key,
    {
        InsertBacktrack {
            builder: self,
            backtrack: vec![],
            new_root: None,
            phantom_key: PhantomData,
        }
    }

    pub(crate) fn delete_node(&mut self, id: PageId) {
        self.old_ids.push(id);
    }

    pub(crate) fn add_new_node(
        &mut self,
        mem_page: crate::mem_page::MemPage,
        key_buffer_size: u32,
    ) -> PageId {
        let id = self.page_manager.new_id();
        let page = Page {
            page_id: id,
            mem_page,
            key_buffer_size,
        };

        // TODO: handle this error
        self.extra.insert(page.page_id, page);
        id
    }

    pub(crate) fn current_root(&self) -> PageId {
        self.current_root
    }

    pub(crate) fn mut_page(&mut self, id: PageId) -> Option<(Option<PageId>, Page)> {
        match self.extra.remove(&id) {
            Some(page) => Some((None, page)),
            None => {
                let page = match self.pages.get_page(id).map(|page| page.get_mut()) {
                    Some(page) => page,
                    None => return None,
                };

                let mut shadow = page;
                let old_id = shadow.page_id;
                shadow.page_id = self.page_manager.new_id();

                Some((Some(old_id), shadow))
            }
        }
    }

    pub(crate) fn add_shadow(&mut self, old_id: PageId, shadow: Page) {
        self.extra.insert(shadow.page_id, shadow);
        self.old_ids.push(old_id);
    }

    pub(crate) fn has_next(&self) -> bool {
        self.current.is_some()
    }

    /// commit creates a new version of the tree, it doesn't sync the file, but it makes the version
    /// available to new readers
    pub(crate) fn commit<K>(mut self)
    where
        K: Key,
    {
        let pages = self.pages;

        for (_id, page) in self.extra.drain() {
            pages.write_page(page).unwrap();
        }

        let transaction = WriteTransaction {
            new_root: self.current_root,
            shadowed_pages: self.old_ids,
            // Pages allocated at the end, basically
            next_page_id: self.page_manager.next_page(),
        };

        let mut current_version = self.current_version.write().unwrap();

        self.versions.push_back(current_version.clone());

        *current_version = Arc::new(Version {
            root: self.current_root,
            transaction,
        });
    }

    // not really needed because the destructor has basically the same effect right now
    pub(crate) fn abort(self) {
        unimplemented!()
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: Key,
{
    pub(crate) fn search_for(&mut self, key: &K) {
        let mut current = self.builder.current_root();

        loop {
            let (old_id, page) = self.builder.mut_page(current).unwrap();

            let found_leaf = page.as_node(|node: Node<K, &[u8]>| {
                if let Some(inode) = node.as_internal() {
                    let upper_pivot = match inode.keys().binary_search(key) {
                        Ok(pos) => Some(pos + 1),
                        Err(pos) => Some(pos),
                    }
                    .filter(|pos| pos < &inode.children().len());

                    if let Some(upper_pivot) = upper_pivot {
                        current = inode.children().get(upper_pivot).unwrap().clone();
                    } else {
                        let last = inode.children().len().checked_sub(1).unwrap();
                        current = inode.children().get(last).unwrap().clone();
                    }
                    false
                } else {
                    true
                }
            });

            self.backtrack.push((old_id, page));

            if found_leaf {
                break;
            }
        }
    }

    pub(crate) fn get_next(&mut self) -> Option<&mut Page> {
        let (old_id, last) = match self.backtrack.pop() {
            Some(pair) => pair,
            None => return None,
        };

        let id = last.page_id;

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        if let Some(old_id) = old_id {
            self.rename_parent(old_id, id);
            self.builder.add_shadow(old_id, last);
        } else {
            self.builder.extra.insert(id, last);
        }

        self.builder.extra.get_mut(&id)
    }

    pub(crate) fn rename_parent(&mut self, old_id: PageId, new_id: PageId) {
        let parent = match self.backtrack.last_mut() {
            Some((_, parent)) => parent,
            None => return,
        };

        parent.as_node_mut(|mut node: Node<K, &mut [u8]>| {
            let mut node = node.as_internal_mut().unwrap();
            let pos_to_update = match node.children().linear_search(&old_id) {
                Some(pos) => pos,
                None => unreachable!(),
            };

            node.children_mut().update(pos_to_update, &new_id).unwrap();
        });
    }

    pub(crate) fn has_next(&self) -> bool {
        self.backtrack.last().is_some()
    }

    pub(crate) fn current_root(&self) -> PageId {
        self.builder.current_root()
    }

    pub(crate) fn add_new_node(&mut self, mem_page: MemPage, key_buffer_size: u32) -> PageId {
        self.builder.add_new_node(mem_page, key_buffer_size)
    }

    pub(crate) fn new_root(&mut self, mem_page: MemPage, key_buffer_size: u32) {
        let id = self.builder.add_new_node(mem_page, key_buffer_size);
        self.new_root = Some(id);
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K> Drop
    for InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: Key,
{
    fn drop(&mut self) {
        while let Some(_) = InsertBacktrack::<'txbuilder, 'txmanager, 'index, K>::get_next(self) {
            ()
        }

        self.builder.current_root = self.new_root.unwrap();
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn active_pages_do_not_get_collected() {
    //     unimplemented!()
    // }
}
