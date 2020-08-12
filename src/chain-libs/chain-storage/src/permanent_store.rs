use crate::Error;
use std::path::Path;

#[derive(Clone)]
pub(crate) struct PermanentStore {
    inner: data_pile::Database,
    id_length: usize,
    chain_length_offset: u32,
}

impl PermanentStore {
    pub fn new<P: AsRef<Path>>(
        path: P,
        id_length: usize,
        chain_length_offset: u32,
    ) -> Result<PermanentStore, Error> {
        let inner = data_pile::Database::new(path)?;

        Ok(Self {
            inner,
            id_length,
            chain_length_offset,
        })
    }

    pub fn get_block_by_chain_length(&self, chain_length: u32) -> Option<data_pile::SharedMmap> {
        let seqno = chain_length - self.chain_length_offset;
        self.inner.get_by_seqno(seqno as usize)
    }

    pub fn put_blocks(&mut self, blocks: &[&[u8]]) -> Result<(), Error> {
        self.inner
            .append(&blocks)
            .map_err(Error::PermanentBackendError)?;

        Ok(())
    }
}
