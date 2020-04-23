use chain_core::property::Block;
use std::collections::HashMap;
use thiserror::Error;

pub(crate) struct ChainStorageIndex<B>
where
    B: Block,
{
    blocks_index: DbIndex<B::Id, RowId>,
    blocks_by_chain_length_index: DbIndex<u64, Vec<B::Id>>,
    block_info_index: DbIndex<B::Id, RowId>,
    tags_index: DbIndex<String, RowId>,
}

#[derive(Debug, Error)]
pub(crate) enum IndexError {
    #[error("this block already exists")]
    BlockExists,
    #[error("could not find block")]
    BlockNotFound,
}

#[derive(Debug, Error)]
pub(crate) enum IndexCreationError {
    #[error("cannot insert to index: {0}")]
    IndexError(#[from] IndexError),
    #[error("cannot read from sqlite: {0}")]
    SQLiteError(#[from] rusqlite::Error),
}

struct DbIndex<K, V>(HashMap<K, V>);

type RowId = isize;

type IndexResult = Result<(), IndexError>;

impl<B> ChainStorageIndex<B>
where
    B: Block,
{
    pub fn new() -> Self {
        Self {
            blocks_index: DbIndex::new(),
            blocks_by_chain_length_index: DbIndex::new(),
            block_info_index: DbIndex::new(),
            tags_index: DbIndex::new(),
        }
    }

    pub fn get_block(&self, key: &B::Id) -> Option<&RowId> {
        self.blocks_index.get(key)
    }

    pub fn get_block_by_chain_length(&self, key: &u64) -> Option<&Vec<B::Id>> {
        self.blocks_by_chain_length_index.get(key)
    }

    pub fn add_block_check(&self, block_id: &B::Id) -> IndexResult {
        if self.blocks_index.get(block_id).is_some()
            || self.block_info_index.get(block_id).is_some()
        {
            return Err(IndexError::BlockExists);
        }
        Ok(())
    }

    pub fn add_block(&mut self, block_id: B::Id, row_id: RowId) -> IndexResult {
        self.add_block_check(&block_id)?;
        self.blocks_index.add(block_id, row_id);

        Ok(())
    }

    pub fn get_block_info(&self, key: &B::Id) -> Option<&RowId> {
        self.block_info_index.get(key)
    }

    pub fn add_block_info(
        &mut self,
        block_id: B::Id,
        chain_length: u64,
        row_id: RowId,
    ) -> IndexResult {
        if self.get_block(&block_id).is_none() {
            return Err(IndexError::BlockNotFound);
        }
        self.block_info_index.add(block_id.clone(), row_id);

        match self.blocks_by_chain_length_index.get_mut(&chain_length) {
            Some(index_row) => index_row.push(block_id),
            None => self
                .blocks_by_chain_length_index
                .add(chain_length, vec![block_id]),
        }

        Ok(())
    }

    pub fn get_tag(&self, tag: &String) -> Option<&RowId> {
        self.tags_index.get(tag)
    }

    pub fn add_tag_check(&self, block_id: &B::Id) -> IndexResult {
        if self.get_block(block_id).is_none() {
            return Err(IndexError::BlockNotFound);
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String, block_id: &B::Id, row_id: RowId) -> IndexResult {
        self.add_tag_check(block_id)?;
        self.tags_index.add(tag, row_id);
        Ok(())
    }
}

impl<K, V> DbIndex<K, V>
where
    K: std::hash::Hash + std::cmp::Eq,
{
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.0.get_mut(key)
    }

    pub fn add(&mut self, key: K, value: V) {
        self.0.insert(key, value);
    }
}
