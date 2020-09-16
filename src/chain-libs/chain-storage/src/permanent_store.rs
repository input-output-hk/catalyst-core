use crate::{BlockInfo, ConsistencyFailure, Error, Value};
use std::path::Path;

#[derive(Clone)]
pub(crate) struct PermanentStore {
    blocks: data_pile::Database,
    chain_length_index: data_pile::Database,
    block_id_index: sled::Tree,
    root_id: Value,
    id_length: usize,
}

impl PermanentStore {
    pub fn file<P: AsRef<Path>, I: Into<Value>>(
        path: P,
        block_id_index: sled::Tree,
        root_id: I,
    ) -> Result<PermanentStore, Error> {
        std::fs::create_dir_all(&path).map_err(Error::Open)?;

        let blocks_path = path.as_ref().join("blocks");
        let chain_length_index_path = path.as_ref().join("chain_length");

        let blocks = data_pile::Database::file(blocks_path)?;
        let chain_length_index = data_pile::Database::file(chain_length_index_path)?;

        let root_id = root_id.into();
        let id_length = root_id.as_ref().len();

        Ok(Self {
            blocks,
            chain_length_index,
            block_id_index,
            root_id,
            id_length,
        })
    }

    pub fn memory<I: Into<Value>>(
        block_id_index: sled::Tree,
        root_id: I,
    ) -> Result<PermanentStore, Error> {
        let blocks = data_pile::Database::memory()?;
        let chain_length_index = data_pile::Database::memory()?;

        let root_id = root_id.into();
        let id_length = root_id.as_ref().len();

        Ok(Self {
            blocks,
            chain_length_index,
            block_id_index,
            root_id,
            id_length,
        })
    }

    pub fn get_block_by_chain_length(&self, chain_length: u32) -> Option<Value> {
        self.blocks
            .get_by_seqno(chain_length as usize)
            .map(Value::permanent)
    }

    pub fn get_block(&self, block_id: &[u8]) -> Result<Option<Value>, Error> {
        self.get_chain_length(block_id).map(|maybe_chain_length| {
            maybe_chain_length.and_then(|chain_length| self.get_block_by_chain_length(chain_length))
        })
    }

    pub fn get_block_info(&self, block_id: &[u8]) -> Result<Option<BlockInfo>, Error> {
        let chain_length = match self.get_chain_length(block_id)? {
            Some(chain_length) => chain_length,
            None => return Ok(None),
        };

        let parent_id = match chain_length.checked_sub(1) {
            Some(chain_length) => Value::permanent(
                self.chain_length_index
                    .get_by_seqno(chain_length as usize)
                    .ok_or(ConsistencyFailure::ChainLength)?,
            ),
            None => self.root_id.clone(),
        };

        let block_id = Value::owned(block_id.to_vec().into_boxed_slice());
        let block_info = BlockInfo::new(block_id, parent_id, chain_length);

        Ok(Some(block_info))
    }

    pub fn get_block_info_by_chain_length(
        &self,
        chain_length: u32,
    ) -> Result<Option<BlockInfo>, Error> {
        let block_id = match self.chain_length_index.get_by_seqno(chain_length as usize) {
            Some(block_id) => block_id,
            None => return Ok(None),
        };

        let parent_id = match chain_length.checked_sub(1) {
            Some(chain_length) => Value::permanent(
                self.chain_length_index
                    .get_by_seqno(chain_length as usize)
                    .ok_or(ConsistencyFailure::ChainLength)?,
            ),
            None => self.root_id.clone(),
        };

        let block_id = Value::permanent(block_id);
        let block_info = BlockInfo::new(block_id, parent_id, chain_length);

        Ok(Some(block_info))
    }

    fn get_chain_length(&self, block_id: &[u8]) -> Result<Option<u32>, Error> {
        let chain_length_bytes_slice = match self.block_id_index.get(block_id)? {
            Some(block_id) => block_id,
            None => return Ok(None),
        };

        let mut chain_length_bytes = [0u8; 4];
        chain_length_bytes.copy_from_slice(chain_length_bytes_slice.as_ref());
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        Ok(Some(chain_length))
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
        assert_eq!(
            ids.len(),
            blocks.len(),
            "the number of ids should be equal to the number of blocks"
        );

        self.blocks
            .append(blocks)
            .map_err(Error::PermanentBackendError)?;

        self.chain_length_index
            .append(ids)
            .map_err(Error::PermanentBackendError)?;

        for (i, id) in ids.iter().enumerate() {
            let chain_length = start_chain_length + i as u32;
            let chain_length_bytes = chain_length.to_le_bytes();
            self.block_id_index.insert(id, &chain_length_bytes[..])?;
        }

        Ok(())
    }

    pub fn iter(&self, chain_length: u32) -> Result<data_pile::SeqNoIter, Error> {
        self.blocks
            .iter_from_seqno(chain_length as usize)
            .ok_or(Error::BlockNotFound)
    }

    pub fn block_id_index(&self) -> &sled::Tree {
        &self.block_id_index
    }
}
