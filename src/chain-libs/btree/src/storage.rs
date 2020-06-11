use memmap::MmapMut;
use std::cell::UnsafeCell;

use parking_lot::RwLock;
use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU64, Ordering};

/// Wrapper over an mmaped file with extension capabilities
/// The underlying file is split into multiple pages, this makes way the pointers(slices) are never invalidated, at the expense of not being to able to
/// operate on chunks spanning multiple pages as contiguous data.
/// The API provided is mostly unsafe, as there is no aliasing checks, it's expected to be done at a higher level.
pub struct MmapStorage {
    pages: ManuallyDrop<PageTable>,
    file_len: AtomicU64,
    allocated_size: AtomicU64,
    file: *mut File,
    page_size: u64,
}

type PageId = u64;

struct PageTable {
    // XXX: A vector may be a decent choice too
    // It may be possible to reduce locking here
    lookup: RwLock<HashMap<PageId, Page>>,
    page_size: u64,
}
struct Page {
    map: UnsafeCell<MmapMut>,
}

impl MmapStorage {
    pub fn new(file: File, page_size: u64) -> Result<Self, io::Error> {
        let file_len = file.metadata()?.len();

        let (page_id, _offset) = absolute_offset_to_relative(page_size, file_len);
        let allocated_size = (page_id + 1) * page_size;

        file.set_len(allocated_size)?;

        let boxed_file = Box::new(file);
        let file = Box::into_raw(boxed_file);

        let pages = ManuallyDrop::new(PageTable {
            lookup: RwLock::new(HashMap::new()),
            page_size,
        });

        Ok(MmapStorage {
            pages,
            file,
            file_len: AtomicU64::new(file_len),
            allocated_size: AtomicU64::new(allocated_size),
            page_size,
        })
    }

    // TODO: Create a better abstraction for chunks that overlap multiple pages?
    /// this panics if the location (+ count) is out of range
    /// the size of the returning slice should be checked as the underlying file is split in multiple pages, it may not be possible to return a slice with the entire range
    ///
    /// # Safety
    ///
    /// This function contains unsafe code: There is no aliasing checking involved. It's up to the caller
    /// to guarantee that there is no mutable borrow overlapping the requested area. The actual cause of the unsafety
    /// is that `get_mut` doesn't take a mutable borrow.
    pub unsafe fn get(&self, location: u64, count: u64) -> &[u8] {
        let (page_id, offset) = absolute_offset_to_relative(self.page_size, location);
        match self.pages.get_page(page_id as PageId) {
            Some(page) => {
                &page[offset..min(offset + count as usize, self.page_size.try_into().unwrap())]
            }
            None => {
                let page = Page::new(
                    memmap::MmapOptions::new()
                        .offset(page_id * self.page_size)
                        .len(self.page_size as usize)
                        .map_mut(&*self.file)
                        .expect("couldn't mmap page"),
                );

                self.pages.add_page(page_id, page);

                self.get(location, count)
            }
        }
    }

    /// caller must enforce that there is no aliasing here
    /// the size of the returning slice should be checked as the underlying file is split in multiple pages, it may not be possible to return a slice with the entire range
    ///
    /// # Safety
    ///
    /// This function contains unsafe code: There is no aliasing checking involved. It's up to the caller
    /// to guarantee that there is no mutable borrow overlapping the requested area.
    pub unsafe fn get_mut(&self, location: u64, count: u64) -> Result<&mut [u8], io::Error> {
        if location + count > self.allocated_size.load(Ordering::SeqCst) {
            self.extend(location + count)?;
        }

        // the file_len is only used in the destructor, to make the file's size on disk be the right one,
        // at that point there is only one instance, so race conditions originated from this are unlikely
        if location + count > self.file_len.load(Ordering::Acquire) {
            self.file_len.store(location + count, Ordering::Release);
        }

        let count: usize = count.try_into().unwrap();

        let (page_id, offset) = absolute_offset_to_relative(self.page_size, location);

        match self.pages.get_page_mut(page_id) {
            Some(page) => {
                Ok(&mut page[offset..min(offset + count, self.page_size.try_into().unwrap())])
            }
            None => {
                let page = memmap::MmapOptions::new()
                    .offset(page_id * self.page_size)
                    .len(self.page_size as usize)
                    .map_mut(&*self.file)
                    .expect("Couldn't map page");

                self.pages.add_page(page_id, Page::new(page));

                Ok(&mut self.pages.get_page_mut(page_id).unwrap()[offset..offset + count as usize])
            }
        }
    }

    fn extend(&self, minimum_required_size: u64) -> Result<(), io::Error> {
        if minimum_required_size > self.allocated_size.load(Ordering::Acquire) {
            let (page_id, _offset) =
                absolute_offset_to_relative(self.page_size, minimum_required_size);
            let new_size = (page_id + 1) * self.page_size;
            // TODO: Is the new expanded section zeroed or something?
            unsafe {
                (&*self.file).set_len(new_size)?;
            }
            self.allocated_size.store(new_size, Ordering::Release);

            self.sync()?;
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        self.pages.sync()
    }

    pub fn len(&self) -> u64 {
        self.file_len.load(Ordering::SeqCst)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Page {
    fn new(map: MmapMut) -> Self {
        Self {
            map: UnsafeCell::new(map),
        }
    }

    fn read(&self) -> *const u8 {
        unsafe { (*self.map.get()).as_ref().as_ptr() }
    }

    unsafe fn write(&self) -> *mut u8 {
        (*self.map.get()).as_mut().as_mut_ptr()
    }

    fn sync(&self) -> Result<(), io::Error> {
        unsafe { (*self.map.get()).flush() }
    }
}

impl PageTable {
    // the caller should ensure to not get a page mutably borrowed
    unsafe fn get_page(&self, id: PageId) -> Option<&[u8]> {
        self.lookup
            .read()
            .get(&id)
            .map(|page| std::slice::from_raw_parts(page.read(), self.page_size.try_into().unwrap()))
    }

    // the caller should ensure to not get a page already borrowed
    unsafe fn get_page_mut(&self, id: PageId) -> Option<&mut [u8]> {
        self.lookup.read().get(&id).map(|page| {
            std::slice::from_raw_parts_mut(page.write(), self.page_size.try_into().unwrap())
        })
    }

    fn sync(&self) -> Result<(), io::Error> {
        for page in self.lookup.read().values() {
            page.sync()?;
        }

        Ok(())
    }

    pub fn add_page(&self, id: PageId, page: Page) {
        self.lookup.write().insert(id, page);
    }
}

impl Drop for MmapStorage {
    fn drop(&mut self) {
        // self.mmap has reference (with an erased lifetime) to the file handle, so we must ensure that it
        // gets dropped first
        unsafe {
            ManuallyDrop::drop(&mut self.pages);
            let file = Box::from_raw(self.file);
            file.set_len(self.file_len.load(Ordering::Acquire)).unwrap();
        }
    }
}

fn absolute_offset_to_relative(page_size: u64, offset: u64) -> (PageId, usize) {
    let page_id = offset / page_size;
    let offset = offset % page_size;
    (page_id, offset.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempfile;

    pub const PAGE_SIZE: u64 = (1 << 20) * 128; // 128mb

    #[test]
    fn map_disjoint_pages() {
        let file = tempfile().unwrap();
        let storage = MmapStorage::new(file, PAGE_SIZE).unwrap();

        let pages = [1u8, 5, 9];
        let mut results = vec![];

        for page in pages.iter() {
            {
                for byte in unsafe { storage.get_mut(*page as u64 * PAGE_SIZE, PAGE_SIZE) }
                    .expect("Couldn't expand file")
                {
                    *byte = *page;
                }
            }
            let result = unsafe { storage.get(*page as u64 * PAGE_SIZE, PAGE_SIZE) };
            results.push((page, result));
        }

        for (page, result) in results {
            assert_eq!(result.len(), PAGE_SIZE as usize);
            // check the first and last elements to make sure that the ranges are mapped correctly
            for b in result.iter().take(10).chain(result.iter().rev().take(10)) {
                assert_eq!(b, page);
            }
        }
    }

    #[test]
    fn non_contiguous_chunk() {
        let file = tempfile().unwrap();
        let storage = MmapStorage::new(file, PAGE_SIZE).unwrap();

        // a chunk of page size starting at page size / 2 should span two pages, so it must be requested in two parts (this means the first part should have page_size / 2 len)
        assert_eq!(
            unsafe { storage.get(PAGE_SIZE / 2, PAGE_SIZE).len() },
            PAGE_SIZE as usize / 2
        );

        assert_eq!(
            unsafe { storage.get_mut(PAGE_SIZE / 2, PAGE_SIZE).unwrap().len() },
            PAGE_SIZE as usize / 2
        );
    }
}
