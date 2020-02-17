use crate::btreeindex::node::Node;
use crate::btreeindex::PageId;
use crate::storage::MmapStorage;
use crate::Key;
use std::marker::PhantomData;

/// An abstraction over a paged file, Pages is kind of an array but backed from disk. Page represents at the moment
/// a heap allocated read/write page, while PageRef is a wrapper to share a read only page in an Arc
/// when we move to mmap, this things may change to take advantage of zero copy.

pub struct Pages {
    storage: MmapStorage,
    page_size: u16,
}

// TODO: move this unsafe impls to MmapStorage? although what is most safe is saying that RwLock<MmapStorage> is Sync + Send
unsafe impl Send for Pages {}
unsafe impl Sync for Pages {}

pub struct PagesInitializationParams {
    pub storage: MmapStorage,
    pub page_size: u16,
    pub key_buffer_size: u32,
}

impl Pages {
    pub fn new(params: PagesInitializationParams) -> Self {
        let PagesInitializationParams {
            storage,
            page_size,
            key_buffer_size: _,
        } = params;

        Pages { storage, page_size }
    }

    pub fn get_page<'a>(&'a self, id: PageId) -> Option<PageHandle<'a, borrow::Immutable>> {
        // TODO: Check the page is actually in range
        // TODO: check mutable aliasing
        let storage = &self.storage;
        let from = u64::from(id.checked_sub(1).expect("0 page is used as a null ptr"))
            * u64::from(self.page_size);

        let page = unsafe { storage.get(from, u64::from(self.page_size)) };
        let handle = PageHandle {
            id,
            borrow: borrow::Immutable { borrow: page },
            page_marker: PhantomData,
        };

        Some(handle)
    }

    pub fn mut_page<'a>(&'a self, id: PageId) -> Result<PageHandle<'a, borrow::Mutable>, ()> {
        // TODO: add checks so the same page is not mutated more than once
        let storage = &self.storage;
        let from = u64::from(id.checked_sub(1).expect("0 page is used as a null ptr"))
            * u64::from(self.page_size);

        // Make sure there is a mapped area for this page
        match unsafe { storage.get_mut(from, u64::from(self.page_size)) } {
            Ok(page) => Ok(PageHandle {
                id,
                borrow: borrow::Mutable { borrow: page },
                page_marker: PhantomData,
            }),
            Err(_) => Err(()),
        }
    }

    pub fn make_shadow(&self, old_id: PageId, new_id: PageId) -> Result<(), ()> {
        assert!(old_id != new_id);
        let page_old = self
            .get_page(old_id)
            .expect("tried to shadow non existing page");

        let mut page_new = self.mut_page(new_id)?;

        page_new.as_slice(|slice| slice.copy_from_slice(page_old.borrow.borrow));

        Ok(())
    }

    pub fn extend(&mut self, to: PageId) -> Result<(), std::io::Error> {
        let storage = &mut self.storage;

        let from = u64::from(to.checked_sub(1).expect("0 page is used as a null ptr"))
            * u64::from(self.page_size);

        storage.resize(from + u64::from(self.page_size))
    }

    pub(crate) fn sync_file(&self) -> Result<(), std::io::Error> {
        self.storage.sync()
    }
}

pub mod borrow {

    pub struct Immutable<'a> {
        pub borrow: &'a [u8],
    }
    pub struct Mutable<'a> {
        pub borrow: &'a mut [u8],
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

impl<'a> PageHandle<'a, borrow::Immutable<'a>> {
    pub fn as_node<K, R>(
        &self,
        _page_size: u64,
        key_buffer_size: usize,
        f: impl FnOnce(Node<K, &[u8]>) -> R,
    ) -> R
    where
        K: Key,
    {
        let page = self.borrow.borrow;

        let node = Node::<K, &[u8]>::from_raw(page.as_ref(), key_buffer_size);

        f(node)
    }
}

impl<'a> PageHandle<'a, borrow::Mutable<'a>> {
    pub fn as_node_mut<K, R>(
        &mut self,
        _page_size: u64,
        key_buffer_size: usize,
        f: impl FnOnce(Node<K, &mut [u8]>) -> R,
    ) -> R
    where
        K: Key,
    {
        let node = Node::<K, &mut [u8]>::from_raw(self.borrow.borrow, key_buffer_size);
        f(node)
    }

    pub fn as_slice(&mut self, f: impl FnOnce(&mut [u8])) {
        f(self.borrow.borrow);
    }
}

#[cfg(test)]
mod test {}
