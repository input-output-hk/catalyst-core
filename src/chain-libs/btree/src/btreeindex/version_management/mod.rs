pub mod transaction;
use super::pages::*;
use super::{transaction::PageRefMut, Metadata, Node, PageId};
use crate::btreeindex::page_manager::PageManager;
use crate::btreeindex::pages::{borrow::Mutable, PageHandle};
use crate::mem_page::MemPage;
use crate::Key;
use std::collections::VecDeque;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
use transaction::{InsertTransaction, MutablePage, ReadTransaction};

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

// this has locks, so no new transaction can occur while this is synced to disk, they are not
// actually used, but they gatekeep new write transactions
pub(crate) struct Checkpoint<'a> {
    pub(crate) new_metadata: Metadata,
    _page_manager: MutexGuard<'a, PageManager>,
    _versions: MutexGuard<'a, VecDeque<Arc<Version>>>,
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
    tx: &'txbuilder mut transaction::InsertTransaction<'txmanager, 'storage>,
    backtrack: Vec<PageId>,
    new_root: Option<PageId>,
    phantom_key: PhantomData<[K]>,
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
    ) -> InsertTransaction<'me, 'index> {
        let page_manager = self.page_manager.lock().unwrap();
        let versions = self.versions.lock().unwrap();

        InsertTransaction::new(
            self.latest_version().root(),
            pages,
            page_manager,
            versions,
            self.latest_version.clone(),
            key_buffer_size,
        )
    }

    /// collect versions without readers, in order to reuse its pages (the ones that are shadow in transactions after that)
    pub fn collect_pending(&self) -> Option<Checkpoint> {
        let mut page_manager = self.page_manager.lock().unwrap();
        let mut versions = self.versions.lock().unwrap();

        let mut pages_to_release = vec![];
        let mut next_page_at_end = None;
        let mut new_root = None;

        // pop versions with only one reference from the front, as those are not reachable anymore
        // also, if there is only one version this will have count of two, because it's also referenced
        // from the current_version member variable. We still need to collect it, otherwise the checkpoint
        // will not include data from the latest write transaction
        while versions.len() > 0 && Arc::strong_count(versions.front().unwrap()) == 1
            || versions.len() == 1 && Arc::strong_count(versions.front().unwrap()) == 2
        {
            // there is no race conditions between the check and this, because versions is locked and count == 1 means is the only reference
            let version = versions.pop_front().unwrap();

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
            _page_manager: page_manager,
            _versions: versions,
        })
    }
}

impl<'txbuilder, 'txmanager: 'txbuilder, 'index: 'txmanager, K>
    InsertBacktrack<'txbuilder, 'txmanager, 'index, K>
where
    K: Key,
{
    /// traverse the tree while storing the path, so we can then backtrack while splitting
    pub fn search_for<'a>(&'a mut self, key: &K) {
        let mut current = self.tx.root();

        loop {
            let page = self.tx.get_page(current).unwrap();

            let found_leaf = page.as_node(
                self.key_buffer_size.try_into().unwrap(),
                |node: Node<K, &[u8]>| {
                    if let Some(inode) = node.try_as_internal() {
                        let upper_pivot = match inode.keys().binary_search(key) {
                            Ok(pos) => Some(pos + 1),
                            Err(pos) => Some(pos),
                        }
                        .filter(|pos| pos < &inode.children().len());

                        if let Some(upper_pivot) = upper_pivot {
                            current = inode.children().get(upper_pivot);
                        } else {
                            let last = inode.children().len().checked_sub(1).unwrap();
                            current = inode.children().get(last);
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

    pub fn get_next<'a>(&'a mut self) -> Result<Option<PageRefMut<'a, 'index>>, std::io::Error> {
        let id = match self.backtrack.pop() {
            Some(id) => id,
            None => return Ok(None),
        };

        if self.backtrack.is_empty() {
            assert!(self.new_root.is_none());
            self.new_root = Some(id);
        }

        let key_buffer_size = usize::try_from(self.key_buffer_size).unwrap();

        match self.tx.mut_page(id)? {
            transaction::MutablePage::NeedsParentRedirect(rename_in_parents) => {
                // this part may be tricky, we need to recursively clone and redirect all the path
                // from the root to the node we are writing to. We need the backtrack stack, because
                // that's the only way to get the parent of a node (because there are no parent pointers)
                // so we iterate it in reverse but without consuming the stack (as we still need it for the
                // rest of the insertion algorithm)
                let mut rename_in_parents = rename_in_parents;
                for id in self.backtrack.iter().rev() {
                    let result = rename_in_parents.rename_parent::<K>(key_buffer_size, *id)?;

                    match result {
                        MutablePage::NeedsParentRedirect(rename) => rename_in_parents = rename,
                        MutablePage::InTransaction(handle) => return Ok(Some(handle)),
                    }
                }
                Ok(Some(rename_in_parents.finish()))
            }
            transaction::MutablePage::InTransaction(handle) => Ok(Some(handle)),
        }
    }

    pub fn has_next(&self) -> bool {
        self.backtrack.last().is_some()
    }

    pub fn add_new_node(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        self.tx.add_new_node(mem_page, key_buffer_size)
    }

    pub fn new_root(
        &mut self,
        mem_page: MemPage,
        key_buffer_size: u32,
    ) -> Result<(), std::io::Error> {
        let id = self.tx.add_new_node(mem_page, key_buffer_size)?;
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
            self.tx.current_root = new_root;
        } else {
            self.tx.current_root = *self.backtrack.first().unwrap();
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
