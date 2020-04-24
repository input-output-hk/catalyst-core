use super::page_manager::PageManager;
use super::{BTreeStoreError, PageId};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::convert::TryInto;
use std::io::{Read, Write};

const MAGIC_SIZE: usize = 8;
const MAGIC: [u8; MAGIC_SIZE] = [0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8];

use super::page_manager::{FIRST_PAGE_ID, NULL_PAGE_ID};

pub(crate) struct StaticSettings {
    pub page_size: u16,
    pub key_buffer_size: u32,
}

impl StaticSettings {
    pub(crate) fn write(&self, writer: &mut impl Write) -> Result<(), BTreeStoreError> {
        writer.write_u32::<LittleEndian>(self.key_buffer_size)?;
        writer.write_u32::<LittleEndian>(self.page_size.into())?;

        Ok(())
    }

    pub(crate) fn read(reader: &mut impl Read) -> Result<StaticSettings, BTreeStoreError> {
        let key_buffer_size = reader.read_u32::<LittleEndian>()?;
        let page_size = reader.read_u32::<LittleEndian>()?;

        Ok(StaticSettings {
            key_buffer_size,
            page_size: page_size.try_into().unwrap(),
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Metadata {
    pub root: PageId,
    pub page_manager: PageManager,
}

impl Metadata {
    pub(crate) fn new() -> Metadata {
        Metadata {
            root: NULL_PAGE_ID,
            page_manager: PageManager {
                free_pages: vec![],
                next_page: FIRST_PAGE_ID,
            },
        }
    }

    pub(crate) fn read(reader: &mut impl Read) -> Result<Metadata, BTreeStoreError> {
        let mut magic = [0u8; MAGIC_SIZE];
        reader.read_exact(&mut magic)?;

        if magic != MAGIC {
            return Err(BTreeStoreError::WrongMagicNumber);
        }

        let root: u32 = reader.read_u32::<LittleEndian>()?;
        let page_manager = PageManager::read(reader)?;

        Ok(Metadata { root, page_manager })
    }

    pub(crate) fn write(&self, writer: &mut impl Write) -> Result<(), BTreeStoreError> {
        writer.write_all(&MAGIC)?;
        writer.write_u32::<LittleEndian>(self.root)?;

        self.page_manager.write(writer)?;

        Ok(())
    }

    pub(crate) fn set_root(&mut self, id: PageId) {
        self.root = id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: use some RAII pattern to clean files after test (it's probably needed somewhere else too)

    use std::fs::OpenOptions;

    #[test]
    fn open_works() {
        let path = "metadata_test";

        {
            let metadata = Metadata::new();
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(path)
                .unwrap();

            metadata.write(&mut file).unwrap();
        }

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .unwrap();
        let metadata = Metadata::read(&mut file).unwrap();

        let page_manager = metadata.page_manager;
        assert_eq!(page_manager.next_page(), FIRST_PAGE_ID);
        assert_eq!(page_manager.free_pages(), &vec![]);

        std::fs::remove_file("metadata_test").unwrap();
    }
}
