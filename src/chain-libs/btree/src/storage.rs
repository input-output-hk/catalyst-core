use memmap::MmapMut;
use std::cell::UnsafeCell;

use std::cmp::min;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

pub struct MmapStorage {
    pages: ManuallyDrop<PageTable>,
    file_len: AtomicU64,
    allocated_size: AtomicU64,
    file: *mut File,
}

type PageId = u64;
const PAGE_SIZE: u64 = (1 << 20) * 128; // 128mb

struct Page {
    map: UnsafeCell<MmapMut>,
}

impl Page {
    fn new(map: MmapMut) -> Self {
        Self {
            map: UnsafeCell::new(map),
        }
    }

    unsafe fn read(&self) -> *const u8 {
        (*self.map.get()).as_ref().as_ptr()
    }

    unsafe fn write(&self) -> *mut u8 {
        (*self.map.get()).as_mut().as_mut_ptr()
    }

    fn sync(&self) -> Result<(), io::Error> {
        unsafe { (*self.map.get()).flush() }
    }
}

struct PageTable {
    // TODO: A vector would be probably be a decent choice too
    lookup: Mutex<HashMap<PageId, Page>>,
}

impl PageTable {
    unsafe fn get_page(&self, id: PageId) -> Option<&[u8]> {
        self.lookup
            .lock()
            .unwrap()
            .get(&id)
            .map(|page| std::slice::from_raw_parts(page.read(), PAGE_SIZE.try_into().unwrap()))
    }

    unsafe fn get_page_mut(&self, id: PageId) -> Option<&mut [u8]> {
        self.lookup
            .lock()
            .unwrap()
            .get(&id)
            .map(|page| std::slice::from_raw_parts_mut(page.write(), PAGE_SIZE.try_into().unwrap()))
    }

    fn sync(&self) -> Result<(), io::Error> {
        for page in self.lookup.lock().unwrap().values() {
            page.sync()?;
        }

        Ok(())
    }

    pub fn add_page(&self, id: PageId, page: Page) {
        self.lookup.lock().unwrap().insert(id, page);
    }
}

impl MmapStorage {
    pub fn new(file: File) -> Result<Self, io::Error> {
        let file_len = file.metadata()?.len();

        let (page_id, _offset) = absolute_offset_to_relative(file_len);
        let allocated_size = (page_id + 1) * PAGE_SIZE;

        file.set_len(allocated_size)?;

        let boxed_file = Box::new(file);
        let file = Box::into_raw(boxed_file);

        let pages = ManuallyDrop::new(PageTable {
            lookup: Mutex::new(HashMap::new()),
        });

        Ok(MmapStorage {
            pages,
            file,
            file_len: AtomicU64::new(file_len),
            allocated_size: AtomicU64::new(allocated_size),
        })
    }

    /// this call is unsafe because get_mut is &self and not &mut self, so this could lead to mutable aliasing
    /// this panics if the location (+ count) is out of range, though
    pub unsafe fn get(&self, location: u64, count: u64) -> &[u8] {
        let (page_id, offset) = absolute_offset_to_relative(location);
        match self.pages.get_page(page_id as PageId) {
            Some(page) => {
                &page[offset..min(offset + count as usize, PAGE_SIZE.try_into().unwrap())]
            }
            None => {
                let page = Page::new(
                    memmap::MmapOptions::new()
                        .offset(page_id * PAGE_SIZE)
                        .len(PAGE_SIZE as usize)
                        .map_mut(&*self.file)
                        .expect("couldn't mmap page"),
                );

                self.pages.add_page(page_id, page);

                self.get(location, count)
            }
        }
    }

    /// caller must enforce that there is no aliasing here
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

        let (page_id, offset) = absolute_offset_to_relative(location);

        match self.pages.get_page_mut(page_id) {
            Some(page) => {
                Ok(&mut page[offset..min(offset + count as usize, PAGE_SIZE.try_into().unwrap())])
            }
            None => {
                let page = memmap::MmapOptions::new()
                    .offset(page_id * PAGE_SIZE)
                    .len(PAGE_SIZE as usize)
                    .map_mut(&*self.file)
                    .expect("Couldn't map page");

                self.pages.add_page(page_id, Page::new(page));

                Ok(&mut self.pages.get_page_mut(page_id).unwrap()[offset..offset + count as usize])
            }
        }
    }

    pub fn extend(&self, minimum_required_size: u64) -> Result<(), io::Error> {
        if minimum_required_size > self.allocated_size.load(Ordering::Acquire) {
            let (page_id, _offset) = absolute_offset_to_relative(minimum_required_size);
            let new_size = (page_id + 1) * PAGE_SIZE;
            // TODO: Is the new expanded section zeroed or something?
            unsafe {
                (&mut *self.file).set_len(new_size)?;
            }
            self.allocated_size.store(new_size, Ordering::Release);

            self.sync()?;
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        // there is nothing really unsafe here, we need the block only because of unsafe cell (at least nothing that is not already present in the memmap api)
        // unsafe { &*self.mmap.get() }.flush()
        self.pages.sync()
    }

    pub fn len(&self) -> u64 {
        self.file_len.load(Ordering::SeqCst)
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

fn absolute_offset_to_relative(offset: u64) -> (PageId, usize) {
    let page_id = offset / PAGE_SIZE;
    let offset = offset % PAGE_SIZE;
    (page_id, offset.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempfile;

    #[test]
    fn mmap_pagination() {
        let file = tempfile().unwrap();
        let mut storage = MmapStorage::new(file).unwrap();

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
        let mut storage = MmapStorage::new(file).unwrap();

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
