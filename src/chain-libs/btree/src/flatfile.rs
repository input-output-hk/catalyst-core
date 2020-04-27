use crate::storage::MmapStorage;

use std::convert::TryInto;
use std::io::{self, Error, ErrorKind, Write};

use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path};

pub const SZ_BITS: usize = 24;
pub const POS_BITS: u64 = 40;
pub const MAX_BLOB_SIZE: usize = 1 << SZ_BITS; // 16MB blob
pub const MAX_POS_OFFSET: u64 = 1 << (POS_BITS - 1); // last possible position 1byte below 1TB

/// Position of a blob in an appender
///
/// The maximum position is defined as MAX_POS_OFFSET
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(u64);

const MAGIC_SIZE: usize = 8;
const MAGIC: [u8; MAGIC_SIZE] = [0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88];
const DATA_START: u64 = 4096;

/// Page size of underlying storage. If a blob can't be stored in the current page, we write it at the beginning of the next page.
/// As this may lead to wasted space, is important to choose the page size accordingly with the expected blob sizes
const MAP_PAGE_SIZE: u64 = (1 << 20) * 128; // 128mb, this allows 8 max size blobs

/// Appender store blob of data (each of maximum size of 16 Mb) offering
/// also a direct access to known index whilst it is appended
pub struct MmapedAppendOnlyFile {
    storage: MmapStorage,
    next_pos: AtomicU64,
}

unsafe impl Send for MmapedAppendOnlyFile {}
unsafe impl Sync for MmapedAppendOnlyFile {}

impl MmapedAppendOnlyFile {
    /// Reopen or create a new appender with the appending file
    pub fn new<P: AsRef<path::Path>>(filename: P) -> Result<Self, io::Error> {
        let filename = filename.as_ref();

        if !filename.exists() {
            let mut f = fs::File::create(&filename)?;
            f.write_all(&MAGIC)?;
            f.set_len(DATA_START)?;
        }

        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&filename)?;

        let storage = MmapStorage::new(file, MAP_PAGE_SIZE)?;
        let next_pos = storage.len();

        unsafe {
            if storage.get(0, MAGIC_SIZE as u64) != MAGIC {
                return Err(Error::new(ErrorKind::Other, "magic mismatch"));
            }
        }

        Ok(Self {
            storage,
            next_pos: AtomicU64::new(next_pos),
        })
    }

    /// Check if this appender can still be appended to.
    pub fn can_append(&self) -> Result<bool, io::Error> {
        let pos = self.next_pos.load(Ordering::SeqCst);
        Ok(pos <= MAX_POS_OFFSET)
    }

    /// Append a blob of data and return the file offset
    ///
    /// Can only append data of MAX_BLOB_SIZE
    pub fn append(&self, buf: &[u8]) -> Result<Pos, io::Error> {
        if buf.len() > MAX_BLOB_SIZE {
            return Err(Error::new(ErrorKind::Other, "blob size too big"));
        }

        // if (buf.len() & 0b11) != 0 {
        //     return Err(Error::new(
        //         ErrorKind::Other,
        //         "blob size is not a multiple of 4",
        //     ));
        // }

        // next_pos is the return value, the mut part is because if there is no space in the current underlying page, we need to move this to the next page boundary
        let mut next_pos = self.next_pos.load(Ordering::Acquire);

        if next_pos > MAX_POS_OFFSET {
            return Err(Error::new(ErrorKind::Other, "offset position too big"));
        }

        let blen = buf.len() as u32;
        let szbuf = blen.to_le_bytes();

        let region_len = szbuf.len() as u64 + buf.len() as u64;

        let mmaped_region = unsafe {
            let region = self.storage.get_mut(next_pos, region_len)?;
            let mapped_len = region.len() as u64;

            // check if we could write everything in a contiguous chunk
            if mapped_len == region_len {
                self.next_pos
                    .store(next_pos + region_len, Ordering::Release);
                region
            } else {
                // if we can't, then we just skip that part and write in the next page and hope it fits
                // we don't write in two different pages in order to just be able to return slices to the mmaped region
                next_pos += mapped_len;
                self.next_pos
                    .store(next_pos + region_len, Ordering::Release);
                self.storage.get_mut(next_pos, region_len)?
            }
        };

        if (mmaped_region.len() as u64) < region_len {
            return Err(Error::new(
                ErrorKind::Other,
                "Couldn't map contiguous region, page size is smaller than blob size",
            ));
        }

        mmaped_region[0..szbuf.len()].copy_from_slice(&szbuf[..]);
        mmaped_region[szbuf.len()..].copy_from_slice(&buf[..]);

        Ok(Pos(next_pos))
    }

    /// Get the blob stored at position @pos
    pub fn get_at(&self, pos: Pos) -> Result<Option<&[u8]>, io::Error> {
        if pos.0 >= self.next_pos.load(Ordering::SeqCst) {
            return Ok(None);
        }

        let szbuf = unsafe { self.storage.get(pos.into(), 4) };

        let len = u32::from_le_bytes(szbuf.try_into().unwrap());

        Ok(Some(unsafe { self.storage.get(pos.0 + 4, len as u64) }))
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        self.storage.sync()?;
        Ok(())
    }
}

impl From<Pos> for u64 {
    fn from(pos: Pos) -> u64 {
        pos.0
    }
}

impl From<u64> for Pos {
    fn from(n: u64) -> Pos {
        Pos(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::rngs::StdRng;
    use rand::Rng as _;
    use rand::SeedableRng;
    use tempfile::tempdir;

    const BLOB_SIZE: usize = 1024;
    const SEED: u64 = 35;

    #[test]
    fn inserted_blobs_are_recoverable() {
        let mut reference = std::collections::BTreeMap::new();

        // the appender does need to create the file, as it applies the initial formatting. This means
        // we can't create a temp file directly, so instead we create a temporal directory and then the
        // appender can create a file inside
        let dir = tempdir().unwrap();
        let mut path = dir.path().to_path_buf();
        path.push("appender");

        let appender = MmapedAppendOnlyFile::new(path).unwrap();

        let mut rng = StdRng::seed_from_u64(SEED);

        for _i in 0..10000 {
            // TODO: use chunks of random sizes too?
            let mut buf = vec![0u8; BLOB_SIZE];
            rng.fill(&mut buf[..]);
            let pos = appender.append(&buf).unwrap();
            reference.insert(pos, buf.into_boxed_slice());
        }

        for (pos, value) in reference.iter() {
            assert_eq!(appender.get_at(*pos).unwrap().unwrap()[..], value[..])
        }
    }

    #[test]
    fn test_need_to_skip_space() {
        // the appender does need to create the file, as it applies the initial formatting. This means
        // we can't create a temp file directly, so instead we create a temporal directory and then the
        // appender can create a file inside
        let dir = tempdir().unwrap();
        let mut path = dir.path().to_path_buf();
        path.push("appender_skip_space");

        let appender = MmapedAppendOnlyFile::new(path).unwrap();

        for _ in 0..(MAP_PAGE_SIZE - 1) / MAX_BLOB_SIZE as u64 {
            let buf = vec![0u8; MAX_BLOB_SIZE];
            appender.append(&buf).unwrap();
        }

        let buf = vec![0u8; MAX_BLOB_SIZE];
        let pos = appender.append(&buf[..]).unwrap();

        assert_eq!(pos.0, MAP_PAGE_SIZE)
    }
}
