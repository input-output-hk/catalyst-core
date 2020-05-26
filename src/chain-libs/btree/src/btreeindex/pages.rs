use crate::btreeindex::node::Node;
use crate::btreeindex::PageId;
use crate::storage::MmapStorage;
use crate::FixedSize;
use std::marker::PhantomData;
use std::sync::Mutex;

/// An abstraction over a paged file, Pages is kind of an array but backed from disk. Page represents at the moment
/// a heap allocated read/write page, while PageRef is a wrapper to share a read only page in an Arc
/// when we move to mmap, this things may change to take advantage of zero copy.

pub struct Pages {
    storage: MmapStorage,
    page_size: u16,
    // we need this just to make the api safe, in general, higher level code shouldn't actually
    // need this checks, as we always clone data before mutating it, and there can only be one transaction
    // at a time, but just in case
    borrows: Mutex<borrow::BorrowChecker>,
}

unsafe impl Send for Pages {}
unsafe impl Sync for Pages {}

pub struct PagesInitializationParams {
    pub storage: MmapStorage,
    pub page_size: u16,
}

impl Pages {
    pub fn new(params: PagesInitializationParams) -> Self {
        let PagesInitializationParams { storage, page_size } = params;

        Pages {
            storage,
            page_size,
            borrows: Mutex::new(borrow::BorrowChecker::new()),
        }
    }

    /// this call is safe, which means that it will panic if the given id is already mutably borrowed
    pub fn get_page<'a>(&'a self, id: PageId) -> Option<PageHandle<'a, borrow::Immutable>> {
        // TODO: Check the page is actually in range
        let borrow_guard = self.borrows.lock().unwrap().borrow(id);

        let storage = &self.storage;
        let from = u64::from(id.checked_sub(1).expect("0 page is used as a null ptr"))
            * u64::from(self.page_size);

        let page = unsafe { storage.get(from, u64::from(self.page_size)) };
        let handle = PageHandle {
            id,
            borrow: borrow::Immutable {
                borrow: page,
                borrow_guard,
            },
            page_marker: PhantomData,
        };

        Some(handle)
    }

    /// this call is safe, which means that it will panic if the given id is already borrowed
    pub fn mut_page<'a>(
        &'a self,
        id: PageId,
    ) -> Result<PageHandle<'a, borrow::Mutable>, std::io::Error> {
        let borrow_guard = self.borrows.lock().unwrap().borrow_mut(id);

        let storage = &self.storage;
        let from = u64::from(id.checked_sub(1).expect("0 page is used as a null ptr"))
            * u64::from(self.page_size);

        // Make sure there is a mapped area for this page
        let page = unsafe { storage.get_mut(from, u64::from(self.page_size))? };
        Ok(PageHandle {
            id,
            borrow: borrow::Mutable {
                borrow: page,
                borrow_guard,
            },
            page_marker: PhantomData,
        })
    }

    /// raw clone page old_id to new_id
    pub fn make_shadow(&self, old_id: PageId, new_id: PageId) -> Result<(), std::io::Error> {
        assert!(old_id != new_id);
        let page_old = self
            .get_page(old_id)
            .expect("tried to shadow non existing page");

        let mut page_new = self.mut_page(new_id)?;

        page_new.as_slice(|slice| slice.copy_from_slice(page_old.borrow.borrow));

        Ok(())
    }

    pub(crate) fn sync_file(&self) -> Result<(), std::io::Error> {
        self.storage.sync()
    }
}

pub mod borrow {
    use crate::btreeindex::PageId;
    use std::collections::HashMap;
    use std::sync::{Arc, Weak};

    pub struct BorrowChecker {
        borrows: HashMap<PageId, Weak<BorrowGuard>>,
    }

    impl BorrowChecker {
        pub fn new() -> BorrowChecker {
            BorrowChecker {
                borrows: HashMap::new(),
            }
        }

        pub fn borrow_mut(&mut self, id: PageId) -> BorrowRAIIGuard {
            let guard = Arc::new(BorrowGuard::Exclusive);

            if self
                .borrows
                .get(&id)
                .and_then(|weak| weak.upgrade())
                .is_some()
            {
                panic!("tried to exclusively borrow already borrowed page");
            }

            self.borrows.insert(id, Arc::downgrade(&guard));

            BorrowRAIIGuard(guard)
        }

        pub fn borrow(&mut self, id: PageId) -> BorrowRAIIGuard {
            use std::cell::RefCell;
            // placeholder to keep the Arc alive while we store the Weak pointer in the map
            let guard = RefCell::new(None);

            let weak = self.borrows.entry(id).or_insert_with(|| {
                // if there is no entry for this id, we allocate one and store the reference
                let mut guard = guard.borrow_mut();
                guard.replace(Arc::new(BorrowGuard::Shared));
                Arc::downgrade(guard.as_ref().unwrap())
            });

            let arc = weak.upgrade().unwrap_or_else(|| {
                // if there was an entry, but it was expired, we need to create and insert a new one
                let mut guard = guard.borrow_mut();
                guard.replace(Arc::new(BorrowGuard::Shared));
                self.borrows
                    .insert(id, Arc::downgrade(guard.as_ref().unwrap()));
                guard.as_ref().unwrap().clone()
            });

            if matches!(*arc, BorrowGuard::Exclusive) {
                panic!("can't borrow mutably borrowed page {}", id)
            }

            BorrowRAIIGuard(arc)
        }
    }

    enum BorrowGuard {
        Shared,
        Exclusive,
    }

    pub struct BorrowRAIIGuard(Arc<BorrowGuard>);

    // TODO: There is nothing to stop anyone from creating an Immutable instance with a Exclusive borrow guard
    pub struct Immutable<'a> {
        pub borrow: &'a [u8],
        pub borrow_guard: BorrowRAIIGuard,
    }
    pub struct Mutable<'a> {
        pub borrow: &'a mut [u8],
        pub borrow_guard: BorrowRAIIGuard,
    }

    impl<'a> Clone for Immutable<'a> {
        fn clone(&self) -> Immutable<'a> {
            Immutable {
                borrow: self.borrow,
                borrow_guard: BorrowRAIIGuard(self.borrow_guard.0.clone()),
            }
        }
    }
}

pub struct PageHandle<'a, Borrow: 'a> {
    id: PageId,
    borrow: Borrow,
    page_marker: PhantomData<&'a Borrow>,
}

impl<'a, T> PageHandle<'a, T> {
    pub fn id(&self) -> PageId {
        self.id
    }
}

use std::clone::Clone;
impl<'a> Clone for PageHandle<'a, borrow::Immutable<'a>> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            borrow: self.borrow.clone(),
            page_marker: PhantomData,
        }
    }
}

impl<'a> PageHandle<'a, borrow::Mutable<'a>> {
    pub fn as_slice(&mut self, f: impl FnOnce(&mut [u8])) {
        f(self.borrow.borrow);
    }
}

impl<'a> super::node::NodeRef for PageHandle<'a, borrow::Immutable<'a>> {
    fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let page = self.borrow.borrow;

        let node = unsafe { Node::<K, &[u8]>::from_raw(page) };

        f(node)
    }
}

impl<'a> super::node::NodeRef for &PageHandle<'a, borrow::Immutable<'a>> {
    fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let page = self.borrow.borrow;

        let node = unsafe { Node::<K, &[u8]>::from_raw(page) };

        f(node)
    }
}

impl<'a> super::node::NodeRef for PageHandle<'a, borrow::Mutable<'a>> {
    fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let node = unsafe { Node::<K, &[u8]>::from_raw(self.borrow.borrow) };

        f(node)
    }
}

impl<'a> super::node::NodeRef for &mut PageHandle<'a, borrow::Mutable<'a>> {
    fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let node = unsafe { Node::<K, &[u8]>::from_raw(self.borrow.borrow) };

        f(node)
    }
}

impl<'a> super::node::NodeRefMut for PageHandle<'a, borrow::Mutable<'a>> {
    fn as_node_mut<K, R>(&mut self, f: impl FnOnce(Node<K, &mut [u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let node = unsafe { Node::<K, &mut [u8]>::from_raw(self.borrow.borrow) };
        f(node)
    }
}

impl<'a> super::node::NodeRefMut for &mut PageHandle<'a, borrow::Mutable<'a>> {
    fn as_node_mut<K, R>(&mut self, f: impl FnOnce(Node<K, &mut [u8]>) -> R) -> R
    where
        K: FixedSize,
    {
        let node = unsafe { Node::<K, &mut [u8]>::from_raw(self.borrow.borrow) };
        f(node)
    }
}

#[cfg(test)]
mod test {}
