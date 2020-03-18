use super::Version;
use crate::btreeindex::{
    borrow::{Immutable, Mutable},
    node::{NodeRef, NodeRefMut},
    page_manager::PageManager,
    Node, PageHandle, PageId, Pages,
};
use crate::Key;
use parking_lot::lock_api;
use parking_lot::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard};
use std::cell::{Cell, RefCell};
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
    pub current_root: Cell<PageId>,
    state: RefCell<State<'locks>>,
    pages: RefCell<ExtendablePages<'storage>>,
    versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
    current_version: Arc<RwLock<Arc<Version>>>,
    key_buffer_size: u32,
}

struct State<'a> {
    /// maps old_id -> new_id
    shadows: HashMap<PageId, PageId>,
    /// in order to find shadows by the new_id (as we already redirected pointers to this)
    shadows_image: HashSet<PageId>,
    deleted_pages: Vec<PageId>,
    page_manager: MutexGuard<'a, PageManager>,
}

#[derive(Clone)]
pub struct PageRef<'a, 'b: 'a> {
    pages: &'a RefCell<ExtendablePages<'b>>,
    page_id: PageId,
}

impl<'a, 'b: 'a> PageRef<'a, 'b> {
    pub fn id(&self) -> PageId {
        self.page_id
    }
}

#[derive(Clone)]
pub struct PageRefMut<'a, 'b: 'a> {
    pages: &'a RefCell<ExtendablePages<'b>>,
    page_id: PageId,
}

impl<'a, 'b: 'a> PageRefMut<'a, 'b> {
    pub fn id(&self) -> PageId {
        self.page_id
    }
}

impl<'a, 'b: 'a> NodeRef for PageRef<'a, 'b> {
    fn as_node<K, R>(&self, key_buffer_size: usize, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: Key,
    {
        self.pages
            .borrow()
            .get_page(self.page_id)
            .expect("page should be already checked")
            .as_node(key_buffer_size, f)
    }
}

impl<'a, 'b: 'a> NodeRef for PageRefMut<'a, 'b> {
    fn as_node<K, R>(&self, key_buffer_size: usize, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: Key,
    {
        self.pages
            .borrow()
            .get_page(self.page_id)
            .expect("page should be already checked")
            .as_node(key_buffer_size, f)
    }
}

impl<'a, 'b: 'a> NodeRefMut for PageRefMut<'a, 'b> {
    fn as_node_mut<K, R>(
        &mut self,
        key_buffer_size: usize,
        f: impl FnOnce(Node<K, &mut [u8]>) -> R,
    ) -> R
    where
        K: Key,
    {
        self.pages
            .borrow()
            .mut_page(self.page_id)
            // FIXME: this unwrap
            .unwrap()
            .as_node_mut(key_buffer_size, f)
    }
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
        let state = State {
            shadows: HashMap::new(),
            shadows_image: HashSet::new(),
            page_manager,
            deleted_pages: Vec::new(),
        };
        InsertTransaction {
            current_root: Cell::new(current_root),
            versions,
            current_version,
            key_buffer_size,
            pages: RefCell::new(ExtendablePages::new(pages)),
            state: RefCell::new(state),
        }
    }

    pub fn root(&self) -> PageId {
        let current_root = self.current_root.get();
        self.state
            .borrow()
            .shadows
            .get(&current_root)
            .map(|root| *root)
            .unwrap_or(current_root)
    }

    pub fn get_page<'this>(&'this self, id: PageId) -> Option<PageRef<'this, 'storage>> {
        let state = self.state.borrow();
        let id = state
            .shadows_image
            .get(&id)
            .or_else(|| state.shadows.get(&id))
            .unwrap_or_else(|| &id);

        let exists = self.pages.borrow().get_page(*id).is_some();

        if exists {
            Some(PageRef {
                pages: &self.pages,
                page_id: *id,
            })
        } else {
            None
        }
    }

    pub fn add_new_node(
        &self,
        mem_page: crate::mem_page::MemPage,
        _key_buffer_size: u32,
    ) -> Result<PageId, std::io::Error> {
        let id = self.state.borrow_mut().page_manager.new_id();

        let pages = self.pages.borrow();
        let result = pages.mut_page(id);

        match result {
            Ok(mut page_handle) => {
                page_handle.as_slice(|page| page.copy_from_slice(mem_page.as_ref()));
            }
            Err(()) => {
                drop(pages);
                self.pages.borrow_mut().extend(id)?;

                let pages = self.pages.borrow();
                // infallible now, after extending the storage
                let mut page_handle = pages.mut_page(id).unwrap();
                page_handle.as_slice(|page| page.copy_from_slice(mem_page.as_ref()));
            }
        };

        Ok(id)
    }

    pub fn delete_node(&self, id: PageId) {
        self.state.borrow_mut().deleted_pages.push(id);
    }

    // TODO: mut_page and mut_page_internal are basically the same thing, but I can't find
    // a straight forward way of reusing the code because of borrowing rules, so I will ignore it for now
    pub fn mut_page<'this>(
        &'this self,
        id: PageId,
    ) -> Result<MutablePage<'this, 'locks, 'storage>, std::io::Error> {
        let mut state = self.state.borrow_mut();

        match state
            .shadows_image
            .get(&id)
            .or_else(|| state.shadows.get(&id))
        {
            Some(id) => {
                let _pre_check = self
                    .pages
                    .borrow()
                    .mut_page(*id)
                    .expect("already fetched page was not allocated");

                let handle = PageRefMut {
                    pages: &self.pages,
                    page_id: *id,
                };

                Ok(MutablePage::InTransaction(handle))
            }
            None => {
                let old_id = id;
                let new_id = state.page_manager.new_id();

                let result = self.pages.borrow().make_shadow(old_id, new_id);

                state.shadows.insert(old_id, new_id);
                state.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.pages.borrow_mut().extend(new_id)?;
                        self.pages.borrow().make_shadow(old_id, new_id).unwrap();
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

    fn mut_page_internal(&self, id: PageId) -> Result<(bool, PageId), std::io::Error> {
        let mut state = self.state.borrow_mut();

        match state
            .shadows_image
            .get(&id)
            .or_else(|| state.shadows.get(&id))
        {
            Some(id) => Ok((false, *id)),
            None => {
                let old_id = id;
                let new_id = state.page_manager.new_id();

                let result = self.pages.borrow().make_shadow(old_id, new_id);

                state.shadows.insert(old_id, new_id);
                state.shadows_image.insert(new_id);

                match result {
                    Ok(()) => (),
                    Err(()) => {
                        self.pages.borrow_mut().extend(new_id)?;
                        // Infallible after extending
                        self.pages.borrow_mut().make_shadow(old_id, new_id).unwrap();
                    }
                }

                Ok((true, new_id))
            }
        }
    }

    /// commit creates a new version of the tree, it doesn't sync the file, but it makes the version
    /// available to new readers
    pub fn commit<K>(mut self)
    where
        K: Key,
    {
        let new_root = self.root();
        let state = self.state.into_inner();
        let transaction = super::WriteTransaction {
            new_root,
            shadowed_pages: state.shadows.keys().cloned().collect(),
            // Pages allocated at the end, basically
            next_page_id: state.page_manager.next_page(),
            deleted_pages: state.deleted_pages,
        };

        let mut current_version = self.current_version.write();

        *current_version = Arc::new(Version {
            root: new_root,
            transaction,
        });

        self.versions.push_back(current_version.clone());
    }
}

pub enum MutablePage<'a, 'b: 'a, 'c: 'b> {
    NeedsParentRedirect(RedirectPointers<'a, 'b, 'c>),
    InTransaction(PageRefMut<'a, 'c>),
}

/// recursive helper for the shadowing process when we need to clone and redirect pointers
pub struct RedirectPointers<'a, 'b: 'a, 'c: 'a> {
    tx: &'a InsertTransaction<'b, 'c>,
    /// id that we need to change in the next step, at some point, we could optimize this to be
    /// an index instead of the id (so we don't need to perform the search)
    last_old_id: PageId,
    last_new_id: PageId,
    /// this is the page that we will return at the end
    shadowed_page: PageId,
}

impl<'a, 'b: 'a, 'c: 'b> RedirectPointers<'a, 'b, 'c> {
    fn find_and_redirect<K: Key, T: NodeRefMut>(&self, key_buffer_size: usize, parent: &mut T) {
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
    }

    // TODO: refactor to merge with rename_parent
    pub fn redirect_parent_in_tx<K: Key>(
        self,
        key_buffer_size: usize,
        mut parent: PageRefMut,
    ) -> PageRefMut<'a, 'c> {
        self.find_and_redirect::<K, PageRefMut>(key_buffer_size, &mut parent);
        self.finish()
    }

    pub fn rename_parent<K: Key>(
        self,
        key_buffer_size: usize,
        parent_id: PageId,
    ) -> Result<MutablePage<'a, 'b, 'c>, std::io::Error> {
        let (parent_needs_shadowing, parent) = self.tx.mut_page_internal(parent_id)?;
        let pages = self.tx.pages.borrow();
        let mut parent = pages.mut_page(parent).unwrap();

        // let old_id = self.last_old_id;
        // let new_id = self.last_new_id;
        // parent.as_node_mut(key_buffer_size, |mut node: Node<K, &mut [u8]>| {
        //     let mut node = node.as_internal_mut();
        //     let pos_to_update = match node.children().linear_search(old_id) {
        //         Some(pos) => pos,
        //         None => unreachable!(),
        //     };

        //     node.children_mut().update(pos_to_update, &new_id).unwrap();
        // });
        self.find_and_redirect::<K, PageHandle<Mutable>>(key_buffer_size, &mut parent);

        let parent_new_id = parent.id();
        if parent_needs_shadowing {
            Ok(MutablePage::NeedsParentRedirect(RedirectPointers {
                tx: self.tx,
                last_old_id: parent_id,
                last_new_id: parent_new_id,
                shadowed_page: self.shadowed_page,
                // this is the parent id of shadowed_page, not of the current node in the iteration
            }))
        } else {
            let page = self.finish();
            Ok(MutablePage::InTransaction(page))
        }
    }

    pub fn finish(self) -> PageRefMut<'a, 'c> {
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

    /// create a staging area for a single insert
    pub(crate) fn delete_backtrack<'me, K>(
        &'me mut self,
    ) -> super::DeleteBacktrack<'me, 'txmanager, 'storage, K>
    where
        K: Key,
    {
        let key_buffer_size = self.key_buffer_size.clone();
        super::DeleteBacktrack {
            tx: self,
            backtrack: vec![],
            parent_info: vec![],
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
