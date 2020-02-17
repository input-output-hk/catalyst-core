use memmap::MmapMut;
use std::cell::UnsafeCell;

use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct MmapStorage {
    mmap: ManuallyDrop<UnsafeCell<MmapMut>>,
    file_len: AtomicU64,
    allocated_size: u64,
    file: *mut File,
}

impl MmapStorage {
    pub fn new(file: File) -> Result<Self, io::Error> {
        let file_len = file.metadata()?.len();

        let allocated_size = Self::next_power_of_two(file_len);
        file.set_len(allocated_size)?;

        let boxed_file = Box::new(file);
        let file = Box::into_raw(boxed_file);
        unsafe {
            Ok(MmapStorage {
                mmap: ManuallyDrop::new(UnsafeCell::new(MmapMut::map_mut(&*file)?)),
                file,
                file_len: AtomicU64::new(file_len),
                allocated_size,
            })
        }
    }

    /// this call is unsafe because get_mut is &self and not &mut self, so this could lead to mutable aliasing
    /// this panics if the location (+ count) is out of range, though
    pub unsafe fn get(&self, location: u64, count: u64) -> &[u8] {
        let location: usize = location.try_into().unwrap();
        let count: usize = count.try_into().unwrap();
        &(*self.mmap.get())[location..location + count]
    }

    /// caller must enforce that there is no aliasing here
    pub unsafe fn get_mut(&self, location: u64, count: u64) -> Result<&mut [u8], u64> {
        if location + count > self.allocated_size {
            return Err(location + count);
        }

        // the file_len is only used in the destructor, to make the file's size on disk be the right one,
        // at that point there is only one instance, so race conditions originated from this are unlikely
        if location + count > self.file_len.load(Ordering::Acquire) {
            self.file_len.store(location + count, Ordering::Release);
        }

        let location: usize = location.try_into().unwrap();
        let count: usize = count.try_into().unwrap();

        // this unwrap can't fail because we already checked for size before
        // I don't think we need any extra synchronization here,
        // at least I don't think get_mut modifies any shared state

        Ok((*self.mmap.get())
            .get_mut(location..location + count)
            .unwrap())
    }

    pub fn extend(&mut self, minimum_required_size: u64) -> Result<(), io::Error> {
        if minimum_required_size > self.allocated_size {
            self.allocated_size = Self::next_power_of_two(minimum_required_size);

            // we need to extend the file, so we unmap, extend and remap
            unsafe {
                // it's really important to flush here
                self.sync()?;
                ManuallyDrop::drop(&mut self.mmap);
                (&mut *self.file).set_len(self.allocated_size)?;
                self.mmap = ManuallyDrop::new(UnsafeCell::new(MmapMut::map_mut(&*self.file)?));
            }
        }
        Ok(())
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        // there is nothing really unsafe here, we need the block only because of unsafe cell (at least nothing that is not already present in the memmap api)
        unsafe { &*self.mmap.get() }.flush()
    }

    fn next_power_of_two(n: u64) -> u64 {
        let mut search = 2;
        while search < n {
            search = search << 2;
        }

        search
    }
}

impl Drop for MmapStorage {
    fn drop(&mut self) {
        // self.mmap has reference (with an erased lifetime) to the file handle, so we must ensure that it
        // gets dropped first
        unsafe {
            ManuallyDrop::drop(&mut self.mmap);
            let file = Box::from_raw(self.file);
            file.set_len(self.file_len.load(Ordering::Acquire)).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempfile;

    #[test]
    fn mmap_put_and_get() {
        let file = tempfile().unwrap();
        let mut storage = MmapStorage::new(file).unwrap();

        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        match unsafe { storage.get_mut(30, 10) } {
            Ok(_) => panic!("Should need resize"),
            Err(pos) => storage.extend(pos).unwrap(),
        }

        unsafe { storage.get_mut(30, 10) }
            .expect("Shouldn't need resize anymore")
            .copy_from_slice(&expected);

        let result = unsafe { storage.get(30, expected.len().try_into().unwrap()) };

        assert_eq!(result, &expected[..]);
    }
}
