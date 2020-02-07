use crate::btreeindex::node::Node;
use crate::btreeindex::PageId;
use crate::storage::{MmapStorage, Storage};
use crate::Key;
use crate::MemPage;
use byteorder::{ByteOrder, LittleEndian};
use std::convert::{TryFrom, TryInto};
use std::sync::{Arc, RwLock};

/// An abstraction over a paged file, Pages is kind of an array but backed from disk. Page represents at the moment
/// a heap allocated read/write page, while PageRef is a wrapper to share a read only page in an Arc
/// when we move to mmap, this things may change to take advantage of zero copy.

#[derive(Clone)]
pub(crate) struct Pages {
    storage: Arc<RwLock<MmapStorage>>,
    page_size: u16,
    // TODO: we need to remove this from here
    key_buffer_size: u32,
}

// TODO: move this unsafe impls to MmapStorage? although what is most safe is saying that RwLock<MmapStorage> is Sync + Send
unsafe impl Send for Pages {}
unsafe impl Sync for Pages {}

pub(crate) struct PagesInitializationParams {
    pub(crate) storage: MmapStorage,
    pub(crate) page_size: u16,
    pub(crate) key_buffer_size: u32,
}

impl Pages {
    pub(crate) fn new(params: PagesInitializationParams) -> Self {
        let PagesInitializationParams {
            storage,
            page_size,
            key_buffer_size,
        } = params;

        let storage = Arc::new(RwLock::new(storage));

        Pages {
            storage,
            page_size,
            key_buffer_size,
        }
    }

    fn read_page(&self, id: PageId) -> MemPage {
        let storage = self.storage.read().unwrap();
        let buf = storage
            .get(
                u64::from(id.checked_sub(1).expect("0 page is used as a null ptr"))
                    * u64::from(self.page_size),
                self.page_size.into(),
            )
            .unwrap();

        let page_size = self.page_size.try_into().unwrap();
        let mut page = MemPage::new(page_size);

        // Ideally, we don't want to make any copies here, but that would require making the mmaped
        // storage thread safe (specially if the mmap gets remapped)
        page.as_mut().copy_from_slice(&buf[..page_size]);

        page
    }

    pub(crate) fn write_page(&self, page: Page) -> Result<(), std::io::Error> {
        let mem_page = &page.mem_page;
        let page_id = page.page_id;

        let mut storage = self.storage.write().unwrap();

        storage
            .put(
                u64::from(page_id.checked_sub(1).unwrap()) * u64::try_from(mem_page.len()).unwrap(),
                &mem_page.as_ref(),
            )
            .unwrap();

        Ok(())
    }

    pub(crate) fn get_page<'a>(&'a self, id: PageId) -> Option<PageRef> {
        // TODO: Check the id is in range?
        let page = self.read_page(id);

        let page_ref = PageRef::new(Page {
            page_id: id,
            key_buffer_size: self.key_buffer_size,
            mem_page: page,
        });

        Some(page_ref.clone())
    }

    pub(crate) fn sync_file(&self) -> Result<(), std::io::Error> {
        self.storage
            .write()
            .expect("Coulnd't acquire tree index lock")
            .sync()
    }
}

use std::fmt;
impl fmt::Debug for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tag = LittleEndian::read_u64(&self.mem_page.as_ref()[0..8]);
        write!(f, "Page {{ page_id: {}, tag: {} }}", self.page_id, tag)
    }
}

pub struct Page {
    pub page_id: PageId,
    pub key_buffer_size: u32,
    pub mem_page: MemPage,
}

#[derive(Clone)]
pub(crate) struct PageRef(Arc<Page>);

unsafe impl Send for PageRef {}
unsafe impl Sync for PageRef {}

impl std::ops::Deref for PageRef {
    type Target = Arc<Page>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Page {
    pub(crate) fn as_node<K, R>(&self, f: impl FnOnce(Node<K, &[u8]>) -> R) -> R
    where
        K: Key,
    {
        let page: &[u8] = self.mem_page.as_ref();
        let node =
            Node::<K, &[u8]>::from_raw(page.as_ref(), self.key_buffer_size.try_into().unwrap());
        f(node)
    }

    pub(crate) fn as_node_mut<K, R>(&mut self, f: impl FnOnce(Node<K, &mut [u8]>) -> R) -> R
    where
        K: Key,
    {
        let page = self.mem_page.as_mut();
        let node = Node::<K, &mut [u8]>::from_raw_mut(
            page.as_mut(),
            self.key_buffer_size.try_into().unwrap(),
        );
        f(node)
    }

    pub(crate) fn id(&self) -> PageId {
        self.page_id
    }
}

impl PageRef {
    pub(crate) fn new(page: Page) -> Self {
        PageRef(Arc::new(page))
    }

    pub(crate) fn get_mut(&self) -> Page {
        let page = &self.0;
        Page {
            page_id: page.page_id,
            key_buffer_size: page.key_buffer_size,
            mem_page: page.mem_page.clone(),
        }
    }
}

#[cfg(test)]
mod test {}
