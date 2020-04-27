use super::Version;
use crate::btreeindex::{
    node::NodeRefMut,
    page_manager::PageManager,
    pages::{
        borrow::{Immutable, Mutable},
        PageHandle,
    },
    Node, PageId, Pages,
};
use crate::FixedSize;
use parking_lot::RwLock;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, MutexGuard};

pub struct ReadTransaction<'a> {
    version: Arc<Version>,
    pages: &'a Pages,
}

/// staging area for batched insertions, it will keep track of pages already shadowed and reuse them,
/// it can be used to create a new `Version` at the end with all the insertions done atomically
pub(crate) struct WriteTransaction<'locks, 'storage: 'locks> {
    pub current_root: Cell<PageId>,
    state: RefCell<State<'locks>>,
    pages: &'storage Pages,
    versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
    current_version: Arc<RwLock<Arc<Version>>>,
}

struct State<'a> {
    /// maps old_id -> new_id
    shadows: HashMap<PageId, PageId>,
    /// in order to find shadows by the new_id (as we already redirected pointers to this)
    shadows_image: HashSet<PageId>,
    deleted_pages: Vec<PageId>,
    page_manager: MutexGuard<'a, PageManager>,
}

pub type PageRef<'a> = PageHandle<'a, Immutable<'a>>;
pub type PageRefMut<'a> = PageHandle<'a, Mutable<'a>>;

impl<'a> ReadTransaction<'a> {
    pub(super) fn new(version: Arc<Version>, pages: &'a Pages) -> ReadTransaction {
        ReadTransaction { version, pages }
    }

    pub fn root(&self) -> PageId {
        self.version.root
    }

    pub fn get_page(&self, id: PageId) -> Option<PageRef<'a>> {
        self.pages.get_page(id)
    }
}

impl<'locks, 'storage: 'locks> WriteTransaction<'locks, 'storage> {
    pub fn new(
        root: PageId,
        pages: &'storage Pages,
        page_manager: MutexGuard<'locks, PageManager>,
        versions: MutexGuard<'locks, VecDeque<Arc<Version>>>,
        current_version: Arc<RwLock<Arc<Version>>>,
    ) -> WriteTransaction<'locks, 'storage> {
        let current_root = root;
        let state = State {
            shadows: HashMap::new(),
            shadows_image: HashSet::new(),
            page_manager,
            deleted_pages: Vec::new(),
        };
        WriteTransaction {
            current_root: Cell::new(current_root),
            versions,
            current_version,
            pages,
            state: RefCell::new(state),
        }
    }

    pub fn root(&self) -> PageId {
        let current_root = self.current_root.get();
        self.state
            .borrow()
            .shadows
            .get(&current_root)
            .copied()
            .unwrap_or(current_root)
    }

    pub fn get_page<'this>(&'this self, id: PageId) -> Option<PageRef<'storage>> {
        let state = self.state.borrow();
        let id = state
            .shadows_image
            .get(&id)
            .or_else(|| state.shadows.get(&id))
            .unwrap_or_else(|| &id);

        self.pages.get_page(*id)
    }

    pub fn add_new_node(
        &self,
        mem_page: crate::mem_page::MemPage,
    ) -> Result<PageId, std::io::Error> {
        let id = self.state.borrow_mut().page_manager.new_id();

        let mut page_handle = self.pages.mut_page(id)?;

        page_handle.as_slice(|page| page.copy_from_slice(mem_page.as_ref()));

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
                let page = self
                    .pages
                    .mut_page(*id)
                    .expect("already fetched page was not allocated");

                Ok(MutablePage::InTransaction(page))
            }
            None => {
                let old_id = id;
                let new_id = state.page_manager.new_id();

                self.pages.make_shadow(old_id, new_id)?;

                state.shadows.insert(old_id, new_id);
                state.shadows_image.insert(new_id);

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

                self.pages.make_shadow(old_id, new_id)?;

                state.shadows.insert(old_id, new_id);
                state.shadows_image.insert(new_id);

                Ok((true, new_id))
            }
        }
    }

    /// commit creates a new version of the tree, it doesn't sync the file, but it makes the version
    /// available to new readers
    pub fn commit<K>(mut self)
    where
        K: FixedSize,
    {
        let new_root = self.root();
        let state = self.state.into_inner();
        let transaction = super::WriteTransactionDelta {
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
    InTransaction(PageRefMut<'c>),
}

/// recursive helper for the shadowing process when we need to clone and redirect pointers
pub struct RedirectPointers<'a, 'b: 'a, 'c: 'a> {
    tx: &'a WriteTransaction<'b, 'c>,
    /// id that we need to change in the next step, at some point, we could optimize this to be
    /// an index instead of the id (so we don't need to perform the search)
    last_old_id: PageId,
    last_new_id: PageId,
    /// this is the page that we will return at the end
    shadowed_page: PageId,
}

impl<'a, 'b: 'a, 'c: 'b> RedirectPointers<'a, 'b, 'c> {
    fn find_and_redirect<K: FixedSize, T: NodeRefMut>(&self, parent: &mut T) {
        let old_id = self.last_old_id;
        let new_id = self.last_new_id;
        parent.as_node_mut(|mut node: Node<K, &mut [u8]>| {
            let mut node = node.as_internal_mut();
            let pos_to_update = match node.children().linear_search(old_id) {
                Some(pos) => pos,
                None => unreachable!(),
            };

            node.children_mut().update(pos_to_update, &new_id).unwrap();
        });
    }

    pub fn redirect_parent_in_tx<K: FixedSize>(self, parent: &mut PageRefMut) -> PageRefMut<'a> {
        self.find_and_redirect::<K, PageRefMut>(parent);
        self.finish()
    }

    pub fn redirect_parent_pointer<K: FixedSize>(
        self,
        parent_id: PageId,
    ) -> Result<MutablePage<'a, 'b, 'c>, std::io::Error> {
        let (parent_needs_shadowing, parent) = self.tx.mut_page_internal(parent_id)?;
        let mut parent = self.tx.pages.mut_page(parent).unwrap();

        self.find_and_redirect::<K, PageHandle<Mutable>>(&mut parent);

        let parent_new_id = parent.id();
        if parent_needs_shadowing {
            Ok(MutablePage::NeedsParentRedirect(RedirectPointers {
                tx: self.tx,
                last_old_id: parent_id,
                last_new_id: parent_new_id,
                shadowed_page: self.shadowed_page,
            }))
        } else {
            let page = self.finish();
            Ok(MutablePage::InTransaction(page))
        }
    }

    pub fn finish(self) -> PageRefMut<'c> {
        match self.tx.mut_page(self.shadowed_page).unwrap() {
            MutablePage::InTransaction(handle) => handle,
            _ => unreachable!(),
        }
    }
}
