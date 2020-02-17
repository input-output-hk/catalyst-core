pub mod transaction;
use super::pages::*;
use super::Metadata;
use super::Node;
use super::PageId;
use crate::btreeindex::page_manager::PageManager;
use crate::btreeindex::pages::{borrow::Mutable, PageHandle};
use crate::mem_page::MemPage;
use crate::Key;

use std::collections::{HashMap, HashSet, VecDeque};
use std::convert::TryInto;
use std::marker::PhantomData;
use transaction::{InsertTransaction, ReadTransaction};

use parking_lot::RwLock;
use std::sync::{Arc, Mutex, MutexGuard};

pub(crate) struct TransactionManager {
    latest_version: Arc<RwLock<Arc<Version>>>,
    versions: Mutex<VecDeque<Arc<Version>>>,
    page_manager: Mutex<PageManager>,
}

#[derive(Debug)]
pub(crate) struct Version {
    root: PageId,
    transaction: WriteTransaction,
}

#[derive(Debug)]
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
    pub fn root(&self) -> PageId {
        self.root
    }
}

/// this is basically a stack, but it will rename pointers and interact with the builder in order to reuse
/// already cloned pages
pub struct InsertBacktrack<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K>
where
    K: Key,
{
    builder: &'txbuilder mut transaction::InsertTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
    page_size: u64,
    key_buffer_size: u32,
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
        self.latest_version.read().clone()
    }

    pub fn read_transaction<'a>(&self, pages: &'a RwLock<Pages>) -> ReadTransaction<'a> {
        let guard = pages.read();
        ReadTransaction::new(self.latest_version(), guard)
    }

    pub fn insert_transaction<'me, 'index: 'me>(
        &'me self,
        pages: &'index RwLock<Pages>,
        key_buffer_size: u32,
        page_size: u64,
    ) -> InsertTransaction<'me, 'index> {
        let page_manager = self.page_manager.lock().unwrap();
        let versions = self.versions.lock().unwrap();

        InsertTransaction {
            current_root: self.latest_version().root(),
            shadows: HashMap::new(),
            shadows_image: HashSet::new(),
            pages: Some(pages.upgradable_read()),
            current: None,
            page_manager,
            versions,
            current_version: self.latest_version.clone(),
            version: self.latest_version(),
            key_buffer_size,
            page_size,
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

        // TODO: this and the loop above are the same thing, but with a special case for the current key
        // which will always have at least two refs, because it's stored separately, remove the duplication

        if versions.len() == 1 && Arc::strong_count(versions.front().unwrap()) == 2 {
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

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: Key,
{
    pub(crate) fn search_for(&mut self, key: &K) {
        let mut current = self.builder.root();

        loop {
            let page = self.builder.get_page(current).unwrap();

            let found_leaf = page.as_node(
                self.page_size,
                self.key_buffer_size.try_into().unwrap(),
                |node: Node<K, &[u8]>| {
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
                },
            );

            self.backtrack.push(page.id());

            if found_leaf {
                break;
            }
        }
    }

    pub(crate) fn get_next(&mut self) -> Result<Option<PageHandle<Mutable>>, std::io::Error> {
        let id = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        let _parent_id = &self.backtrack.last().cloned();

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        let page_size = self.page_size;
        // TODO: Remove as
        let key_buffer_size = self.key_buffer_size as usize;

        use transaction::MutablePage;
        match self.builder.mut_page(id)? {
            transaction::MutablePage::NewShadowingPage(rename_in_parents) => {
                let mut rename_in_parents = rename_in_parents;
                for id in self.backtrack.iter().rev() {
                    let result =
                        rename_in_parents.rename_parent::<K>(page_size, key_buffer_size, *id)?;

                    match result {
                        MutablePage::NewShadowingPage(rename) => rename_in_parents = rename,
                        MutablePage::InTransaction(handle) => return Ok(Some(handle)),
                    }
                }
                Ok(Some(rename_in_parents.finish()))
            }
            transaction::MutablePage::InTransaction(handle) => Ok(Some(handle)),
        }
    }

    pub(crate) fn has_next(&self) -> bool {
        self.backtrack.last().is_some()
    }

    pub(crate) fn current_root(&self) -> PageId {
        self.builder.current_root()
    }

    pub(crate) fn add_new_node(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        self.builder.add_new_node(mem_page, key_buffer_size)
    }

    pub(crate) fn new_root(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<(), std::io::Error> {
        let id = self.builder.add_new_node(mem_page, key_buffer_size)?;
        self.new_root = Some(id);

        Ok(())
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'storage: 'txmanager, K> Drop
    for InsertBacktrack<'txbuilder, 'txmanager, 'storage, K>
where
    K: Key,
{
    fn drop(&mut self) {
        if let Some(new_root) = self.new_root {
            self.builder.current_root = new_root;
        } else {
            self.builder.current_root = *self.backtrack.first().unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    // #[test]
    // fn active_pages_do_not_get_collected() {
    //     unimplemented!()
    // }
}
