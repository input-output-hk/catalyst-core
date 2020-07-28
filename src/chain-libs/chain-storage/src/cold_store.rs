use crate::{block_info::BlockInfo, Error};
use data_pile::{serialization::ConstKeyLenRecordSerializer, Record};
use std::{io::Write, path::Path};

#[derive(Clone)]
pub(crate) struct ColdStore {
    inner: data_pile::Database<ConstKeyLenRecordSerializer>,
    id_length: usize,
}

struct ColdStoreData<'a> {
    pub chain_length: u32,
    pub parent_id: &'a [u8],
    pub data: &'a [u8],
}

impl ColdStore {
    pub fn new<P: AsRef<Path>>(path: P, id_length: usize) -> Result<ColdStore, Error> {
        let serializer = ConstKeyLenRecordSerializer::new(id_length);
        let inner = data_pile::Database::new(path, serializer)?;

        Ok(Self { inner, id_length })
    }

    pub fn block_exists(&self, id: &[u8]) -> bool {
        self.inner.get(id).is_some()
    }

    pub fn get_block(&self, id: &[u8]) -> Option<&[u8]> {
        self.inner
            .get(id)
            .map(|record| ColdStoreData::deserialize(record.value(), self.id_length).data)
    }

    pub fn get_block_info(&self, id: &[u8]) -> Option<BlockInfo> {
        self.inner.get(id).map(|record| {
            ColdStoreData::deserialize(record.value(), self.id_length).block_info(id.to_owned())
        })
    }

    pub fn get_block_by_chain_length(&self, chain_length: u32) -> Option<&[u8]> {
        self.inner
            .get_by_seqno(chain_length as usize)
            .map(|record| ColdStoreData::deserialize(record.value(), self.id_length).data)
    }

    pub fn put_blocks(&self, blocks: &[(Vec<u8>, &BlockInfo)]) -> Result<(), Error> {
        let data: Vec<_> = blocks
            .iter()
            .map(|(data, block_info)| {
                let data =
                    ColdStoreData::new(block_info.chain_length(), block_info.parent_id(), &data);
                let mut serialized_data = Vec::new();
                data.serialize(&mut serialized_data);
                (block_info.id(), serialized_data)
            })
            .collect();

        let records: Vec<_> = data
            .iter()
            .map(|(id, data)| Record::new(id, &data))
            .collect();

        self.inner.append(&records).map_err(Error::ColdBackendError)
    }
}

impl<'a> ColdStoreData<'a> {
    fn new(chain_length: u32, parent_id: &'a [u8], data: &'a [u8]) -> Self {
        Self {
            chain_length,
            parent_id,
            data,
        }
    }

    fn serialize<W: Write>(&self, mut w: W) {
        w.write_all(self.chain_length.to_le_bytes().as_ref())
            .unwrap();
        w.write_all(self.parent_id).unwrap();
        w.write_all(self.data).unwrap();
    }

    fn deserialize(mut r: &'a [u8], id_len: usize) -> Self {
        let mut chain_length_bytes = [0u8; 4];
        chain_length_bytes.copy_from_slice(&r[..4]);
        let chain_length = u32::from_le_bytes(chain_length_bytes);
        r = &r[4..];

        let parent_id = &r[..id_len];
        r = &r[id_len..];

        let data = &r[..];

        Self {
            chain_length,
            parent_id,
            data,
        }
    }

    pub fn block_info<Id: Into<Box<[u8]>>>(&self, id: Id) -> BlockInfo {
        let parent_id = self.parent_id.to_vec();
        BlockInfo::new(id, parent_id, self.chain_length)
    }
}
