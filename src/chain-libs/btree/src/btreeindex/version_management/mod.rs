pub mod transaction;
use super::pages::*;
use super::Metadata;
use crate::btreeindex::page_manager::PageManager;

use super::PageId;

use std::collections::VecDeque;

use transaction::{ReadTransaction, WriteTransaction};

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
    transaction: WriteTransactionDelta,
}

#[derive(Debug)]
/// delta-like structure, it has the list of pages that can be collected after no readers are using them
pub(crate) struct WriteTransactionDelta {
    new_root: PageId,
    shadowed_pages: Vec<PageId>,
    deleted_pages: Vec<PageId>,
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

impl TransactionManager {
    pub fn new(metadata: &Metadata) -> TransactionManager {
        let latest_version = Arc::new(RwLock::new(Arc::new(Version {
            root: metadata.root,
            transaction: WriteTransactionDelta {
                new_root: metadata.root,
                shadowed_pages: vec![],
                next_page_id: metadata.page_manager.next_page(),
                deleted_pages: vec![],
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

    pub fn read_transaction<'a>(&self, pages: &'a Pages) -> ReadTransaction<'a> {
        ReadTransaction::new(self.latest_version(), pages)
    }

    pub fn write_transaction<'me, 'index: 'me>(
        &'me self,
        pages: &'index Pages,
    ) -> WriteTransaction<'me, 'index> {
        let page_manager = self.page_manager.lock().unwrap();
        let versions = self.versions.lock().unwrap();

        WriteTransaction::new(
            self.latest_version().root(),
            pages,
            page_manager,
            versions,
            self.latest_version.clone(),
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

            for id in version.transaction.deleted_pages.iter().cloned() {
                pages_to_release.push(id)
            }

            next_page_at_end = Some(version.transaction.next_page_id);
            new_root = Some(version.transaction.new_root);
        }

        let next_page: PageId = next_page_at_end?;

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

#[cfg(test)]
mod tests {
    // #[test]
    // fn active_pages_do_not_get_collected() {
    //     unimplemented!()
    // }
}
