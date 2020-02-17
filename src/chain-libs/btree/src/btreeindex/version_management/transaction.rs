use super::Version;
use crate::btreeindex::{
    borrow::{Immutable, Mutable},
    page_manager::PageManager,
    Node, PageHandle, PageId, Pages,
};
use crate::Key;
use parking_lot::lock_api;
use parking_lot::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard};
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;
use std::sync::{Arc, MutexGuard};

pub enum MutablePage<'a, 'b: 'a, 'c: 'b> {
    NewShadowingPage(RenamePointers<'a, 'b, 'c>),
    InTransaction(PageHandle<'a, Mutable<'a>>),
}

pub struct RenamePointers<'a, 'b: 'a, 'c: 'a> {
    tx: &'a mut InsertTransaction<'b, 'c>,
    last_old_id: PageId,
    last_new_id: PageId,
    shadowed_page: PageId,
}

impl<'a, 'b: 'a, 'c: 'b> RenamePointers<'a, 'b, 'c> {
    pub fn rename_parent<K: Key>(
        self,
        page_size: u64,
        key_buffer_size: usize,
        parent_id: PageId,
    ) -> Result<MutablePage<'a, 'b, 'c>, std::io::Error> {
        let (parent_needs_shadowing, mut parent) = self.tx.mut_page_unchecked(parent_id)?;

        let old_id = self.last_old_id;
        let new_id = self.last_new_id;
        parent.as_node_mut(
            page_size,
            key_buffer_size,
            |mut node: Node<K, &mut [u8]>| {
                let mut node = node.as_internal_mut().unwrap();
                let pos_to_update = match node.children().linear_search(old_id) {
                    Some(pos) => pos,
                    None => unreachable!(),
                };

                node.children_mut().update(pos_to_update, &new_id).unwrap();
            },
        );

        let parent_new_id = parent.id();
        if parent_needs_shadowing {
            Ok(MutablePage::NewShadowingPage(RenamePointers {
                tx: self.tx,
                last_old_id: parent_id,
                last_new_id: parent_new_id,
                shadowed_page: self.shadowed_page,
            }))
        } else {
            Ok(MutablePage::InTransaction(self.finish()))
        }
    }

    pub fn finish(self) -> PageHandle<'a, Mutable<'a>> {
        match self.tx.mut_page(self.shadowed_page).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            _ => unreachable!(),
        }
    }
}

pub struct ReadTransaction<'a> {
    version: Arc<Version>,
    pages: RwLockReadGuard<'a, Pages>,
}

impl<'a> ReadTransaction<'a> {
    pub(super) fn new(version: Arc<Version>, pages: RwLockReadGuard<Pages>) -> ReadTransaction {
        ReadTransaction { version, pages }
    }

    pub fn root(&self) -> PageId {
        self.version.root
    }

    pub fn get_page(&self, id: PageId) -> Option<PageHandle<Immutable>> {
        self.pages.get_page(id)
    }
}

/// staging area for batched insertions, it will keep track of pages already shadowed and reuse them,
/// it can be used to create a new `Version` at the end with all the insertions done atomically
pub(crate) struct InsertTransaction<'locks, 'storage: 'locks> {
    pub current_root: PageId,
    pub shadows: HashMap<PageId, PageId>,
    pub shadows_image: HashSet<PageId>,
    pub current: Option<usize>,
    pub page_manager: MutexGuard<'locks, PageManager>,
    pub pages: Option<RwLockUpgradableReadGuard<'storage, Pages>>,
    pub versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
    pub current_version: Arc<RwLock<Arc<Version>>>,
    pub version: Arc<Version>,
    pub key_buffer_size: u32,
    pub page_size: u64,
}

impl<'locks, 'storage: 'locks> InsertTransaction<'locks, 'storage> {
    pub fn root(&self) -> PageId {
        self.shadows
            .get(&self.current_root)
            .map(|root| *root)
            .unwrap_or(self.current_root)
    }

    pub fn get_page(&self, id: PageId) -> Option<PageHandle<Immutable>> {
        self.shadows_image
            .get(&id)
            .or_else(|| self.shadows.get(&id))
            .or_else(|| Some(&id))
            .and_then(|id| self.pages.as_ref().unwrap().get_page(*id))
    }

    pub fn add_new_node(
        &mut self,
        mem_page: crate::mem_page::MemPage,
        _key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        let id = self.page_manager.new_id();

        let result = self.pages.as_ref().unwrap().mut_page(id);

        let mut page_handle = match result {
            Ok(page_handle) => page_handle,
            Err(()) => {
                self.extend_storage(id)?;
                // infallible now, after extending the storage
                self.pages.as_ref().unwrap().mut_page(id).unwrap()
            }
        };

        page_handle.as_slice(|page| page.copy_from_slice(mem_page.as_ref()));

        Ok(id)
    }

    pub fn mut_page(
        &mut self,
        id: PageId,
    ) -> Result<MutablePage<'_, 'locks, 'storage>, std::io::Error> {
        match self
            .shadows_image
            .get(&id)
            .or_else(|| self.shadows.get(&id))
        {
            Some(id) => {
                let handle = self
                    .pages
                    .as_ref()
                    .unwrap()
                    .mut_page(*id)
                    .expect("already fetched transaction was not allocated");

                Ok(MutablePage::InTransaction(handle))
            }
            None => {
                let old_id = id;
                let new_id = self.page_manager.new_id();

                let result = self.pages.as_ref().unwrap().make_shadow(old_id, new_id);

                self.shadows.insert(old_id, new_id);
                self.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.extend_storage(new_id)?;
                        self.pages
                            .as_ref()
                            .unwrap()
                            .make_shadow(old_id, new_id)
                            .unwrap();
                    }
                }

                Ok(MutablePage::NewShadowingPage(RenamePointers {
                    tx: self,
                    last_old_id: old_id,
                    last_new_id: new_id,
                    shadowed_page: old_id,
                }))
            }
        }
    }

    fn mut_page_unchecked(
        &mut self,
        id: PageId,
    ) -> Result<(bool, PageHandle<Mutable>), std::io::Error> {
        match self
            .shadows_image
            .get(&id)
            .or_else(|| self.shadows.get(&id))
        {
            Some(id) => {
                let handle = self
                    .pages
                    .as_ref()
                    .unwrap()
                    .mut_page(*id)
                    .expect("already fetched transaction was not allocated");

                Ok((false, handle))
            }
            None => {
                let old_id = id;
                let new_id = self.page_manager.new_id();

                let result = self.pages.as_ref().unwrap().make_shadow(old_id, new_id);

                self.shadows.insert(old_id, new_id);
                self.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.extend_storage(new_id)?;
                        // Infallible after extending
                        self.pages
                            .as_ref()
                            .unwrap()
                            .make_shadow(old_id, new_id)
                            .unwrap();
                    }
                }

                let handle = self.pages.as_ref().unwrap().mut_page(new_id).unwrap();

                Ok((true, handle))
            }
        }
    }

    fn extend_storage(&mut self, including: PageId) -> Result<(), std::io::Error> {
        let mut write_guard =
            lock_api::RwLockUpgradableReadGuard::upgrade(self.pages.take().unwrap());

        (*write_guard).extend(including)?;

        let new_guard = lock_api::RwLockWriteGuard::downgrade_to_upgradable(write_guard);
        self.pages = Some(new_guard);

        Ok(())
    }

    /// commit creates a new version of the tree, it doesn't sync the file, but it makes the version
    /// available to new readers
    pub fn commit<K>(mut self)
    where
        K: Key,
    {
        let transaction = super::WriteTransaction {
            new_root: self.root(),
            shadowed_pages: self.shadows.keys().cloned().collect(),
            // Pages allocated at the end, basically
            next_page_id: self.page_manager.next_page(),
        };

        let mut current_version = self.current_version.write();

        *current_version = Arc::new(Version {
            root: self.root(),
            transaction,
        });

        self.versions.push_back(current_version.clone());
    }
}

impl<'txmanager, 'storage> InsertTransaction<'txmanager, 'storage> {
    /// create a staging area for a single insert
    pub(crate) fn backtrack<'me, K>(
        &'me mut self,
    ) -> super::InsertBacktrack<'me, 'txmanager, 'storage, K>
    where
        K: Key,
    {
        let key_buffer_size = self.key_buffer_size.clone();
        let page_size = self.page_size.clone();
        super::InsertBacktrack {
            builder: self,
            backtrack: vec![],
            new_root: None,
            phantom_key: PhantomData,
            key_buffer_size,
            page_size,
        }
    }

    pub(crate) fn current_root(&self) -> PageId {
        self.current_root
    }
}
