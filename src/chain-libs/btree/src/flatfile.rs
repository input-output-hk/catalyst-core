use crate::storage::MmapStorage;
use byteorder::{ByteOrder, LittleEndian};
use parking_lot::{RwLock, RwLockUpgradableReadGuard, RwLockWriteGuard};
use std::io::{self, Error, ErrorKind, Write};
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path};

pub const SZ_BITS: usize = 24;
pub const POS_BITS: u64 = 40;
pub const MAX_BLOB_SIZE: usize = 2 << SZ_BITS; // 16MB blob
pub const MAX_POS_OFFSET: u64 = 2 << POS_BITS - 1; // last possible position 1byte below 1TB

/// Position of a blob in an appender
///
/// The maximum position is defined as MAX_POS_OFFSET
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(u64);

const MAGIC_SIZE: usize = 8;
const MAGIC: [u8; MAGIC_SIZE] = [0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88];
const DATA_START: u64 = 4096;

/// Appender store blob of data (each of maximum size of 16 Mb) offering
/// also a direct access to known index whilst it is appended
pub struct MmapedAppendOnlyFile {
    storage: RwLock<MmapStorage>,
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

        let storage = MmapStorage::new(file)?;
        let next_pos = storage.len();

        unsafe {
            if storage.get(0, MAGIC_SIZE as u64) != MAGIC {
                return Err(Error::new(ErrorKind::Other, "magic mismatch"));
            }
        }

        Ok(Self {
            storage: RwLock::new(storage),
            next_pos: AtomicU64::new(next_pos),
        })
    }

    /// Check if this appender can still be appended to.
    pub fn can_append(&mut self) -> Result<bool, io::Error> {
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
        if (buf.len() & 0b11) != 0 {
            return Err(Error::new(
                ErrorKind::Other,
                "blob size is not a multiple of 4",
            ));
        }

        let next_pos = self.next_pos.load(Ordering::Acquire);
        let mut storage = self.storage.upgradable_read();

        if next_pos > MAX_POS_OFFSET {
            return Err(Error::new(ErrorKind::Other, "offset position too big"));
        }

        let blen = buf.len() as u32;
        let szbuf = blen.to_le_bytes();

        let region_len = szbuf.len() as u64 + buf.len() as u64;

        let mmaped_region = unsafe {
            match storage.get_mut(next_pos, region_len) {
                Ok(slice) => slice,
                Err(including) => {
                    {
                        let mut new_guard = RwLockUpgradableReadGuard::upgrade(storage);
                        new_guard.extend(including)?;
                        // the upgradable part here is only so we can assign to the storage variable again
                        // we won't upgrade again
                        storage = RwLockWriteGuard::downgrade_to_upgradable(new_guard);
                    }
                    storage.get_mut(next_pos, region_len).unwrap()
                }
            }
        };

        self.next_pos
            .store(next_pos + region_len, Ordering::Release);

        mmaped_region[0..szbuf.len()].copy_from_slice(&szbuf[..]);
        mmaped_region[szbuf.len()..].copy_from_slice(&buf[..]);

        // self.ahandle.sync_data()?;
        Ok(Pos(next_pos))
    }

    /// Get the blob stored at position @pos
    pub fn get_at(&self, pos: Pos) -> Result<Box<[u8]>, io::Error> {
        // TODO: this will panic if position if out of range, we may want to handle that?
        let storage = self.storage.read();
        let szbuf = unsafe { storage.get(pos.into(), 4) };

        let len = LittleEndian::read_u32(&szbuf);

        let mut v = vec![0u8; len as usize];

        unsafe {
            v.copy_from_slice(storage.get(pos.0 + 4, len as u64));
        }

        Ok(v.into())
    }

    pub fn sync(&self) -> Result<(), io::Error> {
        self.storage.read().sync()?;
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
            assert_eq!(appender.get_at(*pos).unwrap()[..], value[..])
        }
    }
}
