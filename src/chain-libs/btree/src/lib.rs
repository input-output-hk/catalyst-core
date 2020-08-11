#[cfg(test)]
#[macro_use]
extern crate quickcheck_macros;

mod arrayview;
pub mod btreeindex;
pub mod flatfile;
mod mem_page;
pub mod storage;
use flatfile::MmapedAppendOnlyFile;

const METADATA_FILE: &str = "metadata";
const TREE_FILE: &str = "pages";
const TREE_SETTINGS_FILE: &str = "settings";
// const BACKUP_FILE: &'static str = "commit_backup";
const APPENDER_FILE_PATH: &str = "flatfile";

use mem_page::MemPage;

use crate::btreeindex::BTree;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::path::Path;

use thiserror::Error;

type Offset = u64;

#[derive(Error, Debug)]
pub enum BTreeStoreError {
    #[error("couldn't create file")]
    IOError(#[from] std::io::Error),
    #[error("invalid directory {0}")]
    InvalidDirectory(&'static str),
    #[error("unknown error")]
    Unknown,
    #[error("duplicated key")]
    DuplicatedKey,
    #[error("key not found")]
    KeyNotFound,
    #[error("wrong magic number")]
    WrongMagicNumber,
    #[error("write implementation not compatible with read")]
    InconsistentWriteRead,
}

pub struct BTreeStore<K>
where
    K: FixedSize,
{
    index: BTree<K, Offset>,
    flatfile: MmapedAppendOnlyFile,
}

impl<K> BTreeStore<K>
where
    K: FixedSize,
{
    pub fn new(
        path: impl AsRef<Path>,
        key_buffer_size: u32,
        page_size: u16,
    ) -> Result<BTreeStore<K>, BTreeStoreError> {
        std::fs::create_dir_all(path.as_ref())?;

        let flatfile = MmapedAppendOnlyFile::new(path.as_ref().join(APPENDER_FILE_PATH))?;

        let tree_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path.as_ref().join(TREE_FILE))?;

        let static_settings_file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path.as_ref().join(TREE_SETTINGS_FILE))?;

        let metadata_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path.as_ref().join(METADATA_FILE))?;

        let index = BTree::<K, Offset>::new(
            metadata_file,
            tree_file,
            static_settings_file,
            page_size,
            key_buffer_size,
        )?;

        Ok(BTreeStore { index, flatfile })
    }

    pub fn open(directory: impl AsRef<Path>) -> Result<BTreeStore<K>, BTreeStoreError> {
        if !directory.as_ref().is_dir() {
            return Err(BTreeStoreError::InvalidDirectory("path is not a directory"));
        }

        let metadata = directory.as_ref().join(METADATA_FILE);

        let file = directory.as_ref().join(TREE_FILE);

        let static_file = directory.as_ref().join(TREE_SETTINGS_FILE);

        let index = BTree::open(metadata, file, static_file)?;

        let mut flatfile = directory.as_ref().to_path_buf();
        flatfile.push(APPENDER_FILE_PATH);

        let appender = MmapedAppendOnlyFile::new(flatfile)?;

        Ok(BTreeStore {
            index,
            flatfile: appender,
        })
    }

    pub fn insert(&self, key: K, blob: &[u8]) -> Result<(), BTreeStoreError> {
        let offset = self.flatfile.append(&blob)?;

        let result = self.index.insert_one(key, offset.into());

        self.flatfile.sync()?;
        self.index.checkpoint()?;

        result
    }

    pub fn delete(&self, key: K) -> Result<(), BTreeStoreError> {
        self.index.delete(&key)?;

        self.flatfile.sync()?;
        self.index.checkpoint()?;

        Ok(())
    }

    /// insert many values in one transaction (with only one fsync)
    pub fn insert_many<B: AsRef<[u8]>>(
        &self,
        iter: impl IntoIterator<Item = (K, B)>,
    ) -> Result<(), BTreeStoreError> {
        let mut offsets: Vec<(K, u64)> = vec![];
        for (key, blob) in iter {
            let offset = self.flatfile.append(blob.as_ref())?;
            offsets.push((key, offset.into()));
        }

        self.index.insert_many(offsets.drain(..))?;

        self.flatfile.sync()?;
        self.index.checkpoint()?;
        Ok(())
    }

    pub fn get(&self, key: &K) -> Result<Option<&[u8]>, BTreeStoreError> {
        self.index
            .get(&key, |offset| offset.cloned())
            .and_then(|pos| {
                self.flatfile
                    .get_at(pos.borrow().clone().into())
                    .transpose()
            })
            .transpose()
            .map_err(|e| e.into())
    }
}

// the reference in this trait is because at some point we could just serve bytes directly as
// references to an mmaped area, and so we could just read the values directly from there (without copies)
// this trait is only used for keys currently, but the idea is to use it both for keys and blobs
pub trait Storeable<'a>: Sized {
    type Error: std::error::Error + Send + Sync;
    type Output: Borrow<Self> + 'a;
    fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn read(buf: &'a [u8]) -> Result<Self::Output, Self::Error>;
}

pub trait FixedSize: for<'a> Storeable<'a> + Ord + Clone + Debug {
    /// max size for an element of this type
    fn max_size() -> usize;
}

impl FixedSize for Offset {
    fn max_size() -> usize {
        std::mem::size_of::<Offset>()
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedSize, Storeable};
    use crate::BTreeStore;
    use byteorder::{ByteOrder, LittleEndian};
    #[derive(Debug, Clone, Ord, Eq, PartialEq, PartialOrd)]
    pub struct U64Key(pub u64);

    impl<'a> Storeable<'a> for U64Key {
        type Error = std::io::Error;
        type Output = Self;

        fn write(&self, buf: &mut [u8]) -> Result<(), Self::Error> {
            LittleEndian::write_u64(buf, self.0);
            Ok(())
        }

        fn read(buf: &'a [u8]) -> Result<Self::Output, Self::Error> {
            Ok(U64Key(LittleEndian::read_u64(buf)))
        }
    }

    impl FixedSize for U64Key {
        fn max_size() -> usize {
            std::mem::size_of::<U64Key>()
        }
    }

    #[test]
    fn is_send() {
        // test (at compile time) that certain types implement the auto-trait Send, either directly for
        // pointer-wrapping types or transitively for types with all Send fields

        fn is_send<T: Send>() {
            // dummy function just used for its parameterized type bound
        }

        is_send::<U64Key>();
        is_send::<BTreeStore<U64Key>>();
    }

    #[test]
    fn is_sync() {
        // test (at compile time) that certain types implement the auto-trait Sync

        fn is_sync<T: Sync>() {
            // dummy function just used for its parameterized type bound
        }

        is_sync::<U64Key>();
        is_sync::<BTreeStore<U64Key>>();
    }
}
