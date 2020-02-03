use memmap::MmapMut;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;

pub trait Storage<'a> {
    type Output;

    fn get(&'a self, location: u64, count: u64) -> Result<Self::Output, io::Error>
    where
        Self::Output: AsRef<[u8]> + 'a;

    fn put(&mut self, location: u64, bytes: impl AsRef<[u8]>) -> Result<(), io::Error>;

    fn sync(&self) -> Result<(), io::Error>;
}

pub struct MmapStorage {
    mmap: ManuallyDrop<MmapMut>,
    file_len: u64,
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
                mmap: ManuallyDrop::new(MmapMut::map_mut(&*file)?),
                file,
                file_len,
                allocated_size,
            })
        }
    }

    fn next_power_of_two(n: u64) -> u64 {
        let mut search = 2;
        while search < n {
            search = search << 2;
        }

        search
    }
}

impl<'a> Storage<'a> for MmapStorage {
    type Output = &'a [u8];

    fn get(&'a self, location: u64, count: u64) -> Result<Self::Output, io::Error> {
        let location: usize = location.try_into().unwrap();
        let count: usize = count.try_into().unwrap();
        Ok(&self.mmap[location..location + count])
    }

    fn put(&mut self, location: u64, bytes: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        if location + u64::try_from(bytes.as_ref().len()).unwrap() > self.allocated_size {
            self.allocated_size = Self::next_power_of_two(
                self.allocated_size + location + u64::try_from(bytes.as_ref().len()).unwrap(),
            );

            // we need to extend the file, so we unmap, extend and remap
            unsafe {
                self.mmap.flush()?;
                ManuallyDrop::drop(&mut self.mmap);
                (&mut *self.file).set_len(self.allocated_size)?;
                self.mmap = ManuallyDrop::new(MmapMut::map_mut(&*self.file)?);
            }
        }

        if location + u64::try_from(bytes.as_ref().len()).unwrap() > self.file_len {
            self.file_len = location + u64::try_from(bytes.as_ref().len()).unwrap();
        }

        let location: usize = location.try_into().unwrap();
        self.mmap
            .get_mut(location..location + bytes.as_ref().len())
            .unwrap()
            .copy_from_slice(&bytes.as_ref());
        Ok(())
    }

    fn sync(&self) -> Result<(), io::Error> {
        self.mmap.flush()
    }
}

impl Drop for MmapStorage {
    fn drop(&mut self) {
        // self.mmap has reference (with an erased lifetime) to the file handle, so we must ensure that it
        // gets dropped first
        unsafe {
            ManuallyDrop::drop(&mut self.mmap);
            let file = Box::from_raw(self.file);
            file.set_len(self.file_len).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;

    struct RAIIFilePath {
        path: String,
    }

    use std::ops::Deref;
    impl Deref for RAIIFilePath {
        type Target = str;
        fn deref(&self) -> &str {
            &self.path
        }
    }

    impl Drop for RAIIFilePath {
        fn drop(&mut self) {
            std::fs::remove_file(&self.path).unwrap();
        }
    }

    #[test]
    fn mmap_put_and_get() {
        let path = RAIIFilePath {
            path: "mmap_put_and_get".to_owned(),
        };
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&*path)
            .unwrap();

        let mut storage = MmapStorage::new(file).unwrap();

        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        storage.put(30, &expected).unwrap();

        let result = storage.get(30, expected.len().try_into().unwrap()).unwrap();

        assert_eq!(result, &expected[..]);
    }
}
