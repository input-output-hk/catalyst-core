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
use std::ops::Deref;
use std::sync::{Arc, MutexGuard};

pub struct ReadTransaction<'a> {
    version: Arc<Version>,
    pages: RwLockReadGuard<'a, Pages>,
}

/// staging area for batched insertions, it will keep track of pages already shadowed and reuse them,
/// it can be used to create a new `Version` at the end with all the insertions done atomically
pub(crate) struct InsertTransaction<'locks, 'storage: 'locks> {
    pub current_root: PageId,
    /// maps old_id -> new_id
    shadows: HashMap<PageId, PageId>,
    /// in order to find shadows by the new_id (as we already redirected pointers to this)
    shadows_image: HashSet<PageId>,
    page_manager: MutexGuard<'locks, PageManager>,
    versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
    pages: ExtendablePages<'storage>,
    current_version: Arc<RwLock<Arc<Version>>>,
    key_buffer_size: u32,
}

// in most cases, we can access the storage with read only access, but in the (rare) cases
// where we need to extend the underlying mmaped file, we need to get exclusive access to the storage
// it is actually not needed for the guard to be atomically upgraded, as there can only be one write
// transaction at a time, but it's useful to not have to implement the drop read guard -> take write guard
// -> drop write guard -> take read guard ourselves
struct ExtendablePages<'storage>(Option<RwLockUpgradableReadGuard<'storage, Pages>>);

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

impl<'locks, 'storage: 'locks> InsertTransaction<'locks, 'storage> {
    pub fn new(
        root: PageId,
        pages: &'storage RwLock<Pages>,
        page_manager: MutexGuard<'locks, PageManager>,
        versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
        current_version: Arc<RwLock<Arc<Version>>>,
        key_buffer_size: u32,
    ) -> InsertTransaction<'locks, 'storage> {
        let current_root = root;
        InsertTransaction {
            current_root,
            page_manager,
            versions,
            current_version,
            key_buffer_size,
            shadows: HashMap::new(),
            shadows_image: HashSet::new(),
            pages: ExtendablePages::new(pages),
        }
    }

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
            .and_then(|id| self.pages.get_page(*id))
    }

    pub fn add_new_node(
        &mut self,
        mem_page: crate::mem_page::MemPage,
        _key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        let id = self.page_manager.new_id();

        let result = self.pages.mut_page(id);

        let mut page_handle = match result {
            Ok(page_handle) => page_handle,
            Err(()) => {
                self.pages.extend(id)?;
                // infallible now, after extending the storage
                self.pages.mut_page(id).unwrap()
            }
        };

        page_handle.as_slice(|page| page.copy_from_slice(mem_page.as_ref()));

        Ok(id)
    }

    // TODO: mut_page and mut_page_internal are basically the same thing, but I can't find
    // a straight forward way of reusing the code because of borrowing rules, so I will ignore it for now
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
                    .mut_page(*id)
                    .expect("already fetched transaction was not allocated");

                Ok(MutablePage::InTransaction(handle))
            }
            None => {
                let old_id = id;
                let new_id = self.page_manager.new_id();

                let result = self.pages.make_shadow(old_id, new_id);

                self.shadows.insert(old_id, new_id);
                self.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.pages.extend(new_id)?;
                        self.pages.make_shadow(old_id, new_id).unwrap();
                    }
                }

                Ok(MutablePage::NeedsParentRedirect(RedirectPointers {
                    tx: self,
                    last_old_id: old_id,
                    last_new_id: new_id,
                    shadowed_page: old_id,
                }))
            }
        }
    }

    fn mut_page_internal(
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
                    .mut_page(*id)
                    .expect("already fetched transaction was not allocated");

                Ok((false, handle))
            }
            None => {
                let old_id = id;
                let new_id = self.page_manager.new_id();

                let result = self.pages.make_shadow(old_id, new_id);

                self.shadows.insert(old_id, new_id);
                self.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.pages.extend(new_id)?;
                        // Infallible after extending
                        self.pages.make_shadow(old_id, new_id).unwrap();
                    }
                }

                let handle = self.pages.mut_page(new_id).unwrap();

                Ok((true, handle))
            }
        }
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

pub enum MutablePage<'a, 'b: 'a, 'c: 'b> {
    NeedsParentRedirect(RedirectPointers<'a, 'b, 'c>),
    InTransaction(PageHandle<'a, Mutable<'a>>),
}

/// recursive helper for the shadowing process when we need to clone and redirect pointers
pub struct RedirectPointers<'a, 'b: 'a, 'c: 'a> {
    tx: &'a mut InsertTransaction<'b, 'c>,
    /// id that we need to change in the next step, at some point, we could optimize this to be
    /// an index instead of the id (so we don't need to perform the search)
    last_old_id: PageId,
    last_new_id: PageId,
    /// this is the page that we will return at the end
    shadowed_page: PageId,
}

impl<'a, 'b: 'a, 'c: 'b> RedirectPointers<'a, 'b, 'c> {
    pub fn rename_parent<K: Key>(
        self,
        key_buffer_size: usize,
        parent_id: PageId,
    ) -> Result<MutablePage<'a, 'b, 'c>, std::io::Error> {
        let (parent_needs_shadowing, mut parent) = self.tx.mut_page_internal(parent_id)?;

        let old_id = self.last_old_id;
        let new_id = self.last_new_id;
        parent.as_node_mut(key_buffer_size, |mut node: Node<K, &mut [u8]>| {
            let mut node = node.as_internal_mut();
            let pos_to_update = match node.children().linear_search(old_id) {
                Some(pos) => pos,
                None => unreachable!(),
            };

            node.children_mut().update(pos_to_update, &new_id).unwrap();
        });

        let parent_new_id = parent.id();
        if parent_needs_shadowing {
            Ok(MutablePage::NeedsParentRedirect(RedirectPointers {
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

impl<'txmanager, 'storage> InsertTransaction<'txmanager, 'storage> {
    /// create a staging area for a single insert
    pub(crate) fn backtrack<'me, K>(
        &'me mut self,
    ) -> super::InsertBacktrack<'me, 'txmanager, 'storage, K>
    where
        K: Key,
    {
        let key_buffer_size = self.key_buffer_size.clone();
        super::InsertBacktrack {
            tx: self,
            backtrack: vec![],
            new_root: None,
            phantom_key: PhantomData,
            key_buffer_size,
        }
    }
}

impl<'storage> Deref for ExtendablePages<'storage> {
    type Target = Pages;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<'storage> ExtendablePages<'storage> {
    fn new(pages: &'storage RwLock<Pages>) -> ExtendablePages<'storage> {
        Self(Some(pages.upgradable_read()))
    }

    fn extend(&mut self, to: PageId) -> Result<(), std::io::Error> {
        let mut write_guard = lock_api::RwLockUpgradableReadGuard::upgrade(self.0.take().unwrap());

        (*write_guard).extend(to)?;

        let new_guard = lock_api::RwLockWriteGuard::downgrade_to_upgradable(write_guard);
        self.0 = Some(new_guard);

        Ok(())
    }
}
