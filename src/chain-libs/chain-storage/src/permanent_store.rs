use crate::{Error, Value};
use std::path::Path;

#[derive(Clone)]
pub(crate) struct PermanentStore {
    inner: data_pile::Database,
    block_id_index: sled::Tree,
    id_length: usize,
    chain_length_offset: u32,
}

impl PermanentStore {
    pub fn new<P: AsRef<Path>>(
        path: P,
        block_id_index: sled::Tree,
        id_length: usize,
        chain_length_offset: u32,
    ) -> Result<PermanentStore, Error> {
        let inner = data_pile::Database::new(path)?;

        Ok(Self {
            inner,
            block_id_index,
            id_length,
            chain_length_offset,
        })
    }

    pub fn get_block_by_chain_length(&self, chain_length: u32) -> Option<Value> {
        let seqno = chain_length - self.chain_length_offset;
        self.inner
            .get_by_seqno(seqno as usize)
            .map(Value::permanent)
    }

    pub fn get_block(&self, block_id: &[u8]) -> Result<Option<Value>, Error> {
        let chain_length_bytes_slice = match self.block_id_index.get(block_id)? {
            Some(block_id) => block_id,
            None => return Ok(None),
        };

        let mut chain_length_bytes = [0u8; 4];
        chain_length_bytes.copy_from_slice(chain_length_bytes_slice.as_ref());
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        Ok(self.get_block_by_chain_length(chain_length))
    }

    pub fn contains_key(&self, block_id: &[u8]) -> Result<bool, Error> {
        self.block_id_index
            .contains_key(block_id)
            .map_err(Into::into)
    }

    pub fn put_blocks(
        &self,
        start_chain_length: u32,
        ids: &[&[u8]],
        blocks: &[&[u8]],
    ) -> Result<(), Error> {
        assert_eq!(ids.len(), blocks.len());

        self.inner
            .append(blocks)
            .map_err(Error::PermanentBackendError)?;

        for (i, id) in ids.iter().enumerate() {
            let chain_length = start_chain_length + i as u32;
            let chain_length_bytes = chain_length.to_le_bytes();
            self.block_id_index.insert(id, &chain_length_bytes[..])?;
        }

        Ok(())
    }

    pub fn iter(&self, chain_length: u32) -> Result<data_pile::SeqNoIter, Error> {
        let seqno = chain_length - self.chain_length_offset;
        self.inner
            .iter_from_seqno(seqno as usize)
            .ok_or(Error::BlockNotFound)
    }
}
