mod block_info;
mod cold_store;
#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils;
mod value;

use cold_store::ColdStore;
use sled::{ConflictableTransactionError, TransactionError, Transactional, TransactionalTree};
use std::path::Path;
use thiserror::Error;

pub use block_info::BlockInfo;
pub use value::Value;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to open the database directory")]
    Open(#[source] std::io::Error),
    #[error("block not found")]
    BlockNotFound,
    #[error("database backend error")]
    HotBackendError(#[from] sled::Error),
    #[error("cold store error")]
    ColdBackendError(#[from] data_pile::Error),
    #[error("Block already present in DB")]
    BlockAlreadyPresent,
    #[error("the parent block is missing for the required write")]
    MissingParent,
    #[error("branch with the requested tip does not exist")]
    BranchNotFound,
}

#[derive(Clone)]
pub struct BlockStore {
    hot: sled::Db,
    cold: ColdStore,
    root_id: Value,
    id_length: usize,
}

enum RemoveTipResult {
    NextTip(Vec<u8>),
    HitColdStore(Vec<u8>),
    Done,
}

mod tree {
    pub const BLOCKS: &str = "blocks";
    pub const COLD_STORE_BLOCKS: &str = "cold_store";
    pub const INFO: &str = "info";
    pub const CHAIN_HEIGHT_INDEX: &str = "height_to_block_ids";
    pub const BRANCHES_TIPS: &str = "branches_tips";
    pub const TAGS: &str = "tags";
}

impl BlockStore {
    pub fn new<P: AsRef<Path>, I: Into<Value>>(
        path: P,
        root_id: I,
        id_length: usize,
        chain_length_offset: u32,
    ) -> Result<Self, Error> {
        if !path.as_ref().exists() {
            std::fs::create_dir(path.as_ref()).map_err(Error::Open)?;
        }

        let hot_path = path.as_ref().join("hot");
        let cold_path = path.as_ref().join("cold");

        let hot = sled::open(hot_path)?;
        let cold = ColdStore::new(cold_path, id_length, chain_length_offset)?;

        let root_id = root_id.into();

        Ok(Self {
            hot,
            cold,
            root_id,
            id_length,
        })
    }

    /// Write a block to the store. The parent of the block must exist (unless
    /// it's the zero hash).
    ///
    /// # Arguments
    ///
    /// * `block` - a serialized representation of a block.
    /// * `block_info` - block metadata for internal needs (indexing, linking
    ///   between blocks, etc)
    pub fn put_block(&mut self, block: &[u8], block_info: BlockInfo) -> Result<(), Error> {
        let blocks = self.hot.open_tree(tree::BLOCKS)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;
        let info = self.hot.open_tree(tree::INFO)?;
        let height_to_block_ids = self.hot.open_tree(tree::CHAIN_HEIGHT_INDEX)?;
        let tips = self.hot.open_tree(tree::BRANCHES_TIPS)?;

        let result = (
            &blocks,
            &cold_store_blocks,
            &info,
            &height_to_block_ids,
            &tips,
        )
            .transaction(
                |(blocks, cold_store_blocks, info, height_to_block_ids, tips)| {
                    put_block_impl(
                        blocks,
                        cold_store_blocks,
                        info,
                        height_to_block_ids,
                        tips,
                        block,
                        &block_info,
                        self.root_id.as_ref(),
                        self.id_length,
                    )
                },
            );

        convert_transaction_result(result)
    }

    /// Get a block from the storage.
    ///
    /// # Arguments
    ///
    /// * `block_id` - the serialized block identifier.
    pub fn get_block(&mut self, block_id: &[u8]) -> Result<Value, Error> {
        let blocks = self.hot.open_tree(tree::BLOCKS)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;

        if let Some(chain_length_bytes_slice) = cold_store_blocks.get(block_id)? {
            let mut chain_length_bytes = [0u8; 4];
            chain_length_bytes.copy_from_slice(chain_length_bytes_slice.as_ref());
            let chain_length = u32::from_le_bytes(chain_length_bytes);

            return self
                .cold
                .get_block_by_chain_length(chain_length)
                .map(Value::shared)
                .ok_or(Error::BlockNotFound);
        }

        blocks
            .get(block_id)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(Value::volatile)
    }

    /// Get the `BlockInfo` instance for the requested block.
    ///
    /// # Arguments
    ///
    /// * `block_id` - the serialized block identifier.
    pub fn get_block_info(&mut self, block_hash: &[u8]) -> Result<BlockInfo, Error> {
        let info = self.hot.open_tree(tree::INFO)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;
        let chain_height_index = self.hot.open_tree(tree::CHAIN_HEIGHT_INDEX)?;

        if let Some(chain_length_bytes_slice) = cold_store_blocks.get(block_hash)? {
            let mut chain_length_bytes = [0u8; 4];
            chain_length_bytes.copy_from_slice(chain_length_bytes_slice.as_ref());
            let chain_length = u32::from_le_bytes(chain_length_bytes);

            let parent_id = chain_length
                .checked_sub(1)
                .map(|parent_chain_length| parent_chain_length.to_le_bytes())
                .and_then(|parent_chain_length_bytes| {
                    chain_height_index
                        .scan_prefix(&parent_chain_length_bytes[..])
                        .next()
                })
                .transpose()?
                .map(|(key, _)| key[4..].to_vec().into())
                .unwrap_or_else(|| self.root_id.clone());

            let block_info = BlockInfo::new(block_hash.to_vec(), parent_id, chain_length);

            return Ok(block_info);
        }

        info.get(block_hash)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader, self.id_length, block_hash.to_vec())
            })
    }

    /// Get multiple serialized blocks from the given height.
    pub fn get_blocks_by_chain_length(&mut self, chain_length: u32) -> Result<Vec<Value>, Error> {
        if let Some(block) = self.cold.get_block_by_chain_length(chain_length) {
            return Ok(vec![Value::shared(block)]);
        }

        let blocks = self.hot.open_tree(tree::BLOCKS)?;
        let height_to_block_ids = self.hot.open_tree(tree::CHAIN_HEIGHT_INDEX)?;

        let height_index_prefix = chain_length.to_le_bytes();
        height_to_block_ids
            .scan_prefix(height_index_prefix)
            .map(|scan_result| {
                let block_hash = scan_result.map(|(key, _)| Vec::from(&key[4..key.len()]))?;

                blocks
                    .get(block_hash)
                    .map(|maybe_raw_block| Value::volatile(maybe_raw_block.unwrap()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Add a tag for a given block. The block id can be later retrieved by this
    /// tag.
    pub fn put_tag(&mut self, tag_name: &str, block_hash: &[u8]) -> Result<(), Error> {
        let info = self.hot.open_tree(tree::INFO)?;
        let tags = self.hot.open_tree(tree::TAGS)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;

        let result = (&info, &tags, &cold_store_blocks).transaction(
            move |(info, tags, cold_store_blocks)| {
                put_tag_impl(
                    info,
                    tags,
                    cold_store_blocks,
                    tag_name,
                    block_hash,
                    self.id_length,
                )
            },
        );

        convert_transaction_result(result)
    }

    /// Get the block id for the given tag.
    pub fn get_tag(&mut self, tag_name: &str) -> Result<Option<Value>, Error> {
        let tags = self.hot.open_tree(tree::TAGS)?;

        tags.get(tag_name)
            .map(|maybe_id_bin| maybe_id_bin.map(Value::volatile))
            .map_err(Into::into)
    }

    /// Get identifier of all branches tips.
    pub fn get_tips_ids(&mut self) -> Result<Vec<Value>, Error> {
        let tips = self.hot.open_tree(tree::BRANCHES_TIPS)?;

        tips.iter()
            .map(|id_result| id_result.map(|(id, _)| Value::volatile(id)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Prune a branch with the given tip id from the storage.
    pub fn prune_branch(&mut self, tip_id: &[u8]) -> Result<(), Error> {
        let tips = self.hot.open_tree(tree::BRANCHES_TIPS)?;

        if !tips.contains_key(tip_id)? {
            return Err(Error::BranchNotFound);
        }

        let blocks = self.hot.open_tree(tree::BLOCKS)?;
        let info = self.hot.open_tree(tree::INFO)?;
        let height_to_block_ids = self.hot.open_tree(tree::CHAIN_HEIGHT_INDEX)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;

        let result = (
            &blocks,
            &info,
            &height_to_block_ids,
            &tips,
            &cold_store_blocks,
        )
            .transaction(
                |(blocks, info, height_to_block_ids, tips, cold_store_blocks)| {
                    let mut result = RemoveTipResult::NextTip(Vec::from(tip_id));

                    while let RemoveTipResult::NextTip(current_tip) = &result {
                        result = remove_tip_impl(
                            blocks,
                            info,
                            height_to_block_ids,
                            tips,
                            cold_store_blocks,
                            current_tip,
                            self.root_id.as_ref(),
                            self.id_length,
                        )?;
                    }

                    Ok(Ok(result))
                },
            );

        let result = convert_transaction_result(result)?;

        if let RemoveTipResult::HitColdStore(id) = result {
            let block_info = self
                .get_block_info(&id)
                .expect("parent block in cold store not found");
            let chain_length = block_info.chain_length() + 1;

            if self.get_blocks_by_chain_length(chain_length)?.is_empty() {
                tips.insert(block_info.id(), &[])?;
            }
        }

        Ok(())
    }

    /// Check if the block with the given id exists.
    pub fn block_exists(&mut self, block_hash: &[u8]) -> Result<bool, Error> {
        let info = self.hot.open_tree(tree::INFO)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;

        if cold_store_blocks.get(block_hash)?.is_some() {
            return Ok(true);
        }

        info.get(block_hash)
            .map(|maybe_block| maybe_block.is_some())
            .map_err(Into::into)
    }

    /// Determine whether block 'ancestor' is an ancestor of block 'descendent'
    ///
    /// Returned values:
    /// - `Ok(Some(dist))` - `ancestor` is ancestor of `descendent`
    ///     and there are `dist` blocks between them
    /// - `Ok(None)` - `ancestor` is not ancestor of `descendent`
    /// - `Err(error)` - `ancestor` or `descendent` was not found
    pub fn is_ancestor(
        &mut self,
        ancestor_id: &[u8],
        descendent_id: &[u8],
    ) -> Result<Option<u32>, Error> {
        let descendent = self.get_block_info(descendent_id)?;

        if ancestor_id == descendent_id {
            return Ok(Some(0));
        }

        if ancestor_id == self.root_id.as_ref() {
            return Ok(Some(descendent.chain_length()));
        }

        let ancestor = self.get_block_info(ancestor_id)?;

        if ancestor.chain_length() >= descendent.chain_length() {
            return Ok(None);
        }

        if descendent.parent_id() == ancestor.id() {
            return Ok(Some(1));
        }

        let mut current_block_info = descendent;
        let mut distance = 0;

        while let Some(parent_block_info) = self
            .get_block_info(current_block_info.parent_id().as_ref())
            .map(Some)
            .or_else(|err| match err {
                Error::BlockNotFound => Ok(None),
                e => Err(e),
            })?
        {
            distance += 1;
            if parent_block_info.id().as_ref() == ancestor_id {
                return Ok(Some(distance));
            }
            current_block_info = parent_block_info;
        }

        Ok(None)
    }

    pub fn get_nth_ancestor(
        &mut self,
        block_hash: &[u8],
        distance: u32,
    ) -> Result<BlockInfo, Error> {
        for_path_to_nth_ancestor(self, block_hash, distance, |_| {})
    }

    pub fn flush_to_cold_store(&mut self, to_block: &[u8]) -> Result<(), Error> {
        let mut block_infos = Vec::new();

        let mut current_block_hash = to_block;

        while let Some(block_info) =
            self.get_block_info(current_block_hash)
                .map(Some)
                .or_else(|err| match err {
                    Error::BlockNotFound => Ok(None),
                    e => Err(e),
                })?
        {
            block_infos.push(block_info);
            current_block_hash = block_infos.last().unwrap().parent_id().as_ref();
        }

        block_infos.reverse();

        let blocks = block_infos
            .iter()
            .map(|block_info| self.get_block(block_info.id().as_ref()))
            .collect::<Result<Vec<_>, Error>>()?;
        let block_refs: Vec<_> = blocks.iter().map(|block| block.as_ref()).collect();

        self.cold.put_blocks(&block_refs)?;

        let blocks = self.hot.open_tree(tree::BLOCKS)?;
        let info = self.hot.open_tree(tree::INFO)?;
        let cold_store_blocks = self.hot.open_tree(tree::COLD_STORE_BLOCKS)?;

        for block_info in block_infos.iter() {
            let key = block_info.id();
            let chain_length_bytes = block_info.chain_length().to_le_bytes();

            cold_store_blocks.insert(key, chain_length_bytes.as_ref())?;
            blocks.remove(key)?;
            info.remove(key)?;
        }

        Ok(())
    }
}

/// Like `BlockStore::get_nth_ancestor`, but calls the closure 'callback' with
/// each intermediate block encountered while travelling from
/// 'block_hash' to its n'th ancestor.
pub fn for_path_to_nth_ancestor<F>(
    store: &mut BlockStore,
    block_hash: &[u8],
    distance: u32,
    mut callback: F,
) -> Result<BlockInfo, Error>
where
    F: FnMut(&BlockInfo),
{
    let mut current = store.get_block_info(block_hash)?;

    if distance >= current.chain_length() {
        panic!(
            "distance {} > chain length {}",
            distance,
            current.chain_length()
        );
    }

    let target = current.chain_length() - distance;

    while target < current.chain_length() {
        callback(&current);
        current = store.get_block_info(current.parent_id().as_ref())?;
    }

    Ok(current)
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn put_block_impl(
    blocks: &TransactionalTree,
    cold_store_blocks: &TransactionalTree,
    info: &TransactionalTree,
    height_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    block: &[u8],
    block_info: &BlockInfo,
    root_id: &[u8],
    id_length: usize,
) -> Result<Result<(), Error>, ConflictableTransactionError<()>> {
    if info.get(block_info.id())?.is_some() || cold_store_blocks.get(block_info.id())?.is_some() {
        return Ok(Err(Error::BlockAlreadyPresent));
    }

    let parent_in_cold_store = cold_store_blocks.get(block_info.id())?.is_some();

    if block_info.parent_id().as_ref() != root_id {
        if info.get(block_info.parent_id())?.is_none() && !parent_in_cold_store {
            return Ok(Err(Error::MissingParent));
        }

        if !parent_in_cold_store {
            let parent_block_info_bin = info.get(block_info.parent_id())?.unwrap();
            let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
            let mut parent_block_info = BlockInfo::deserialize(
                &mut parent_block_info_reader,
                id_length,
                block_info.parent_id().clone(),
            );
            parent_block_info.add_ref();
            info.insert(
                parent_block_info.id().as_ref(),
                parent_block_info.serialize(),
            )?;
        }
    }

    tips.remove(block_info.parent_id().as_ref())?;
    tips.insert(block_info.id().as_ref(), &[])?;

    let mut height_index = block_info.chain_length().to_le_bytes().to_vec();
    height_index.extend_from_slice(block_info.id().as_ref());
    height_to_block_ids.insert(height_index, &[])?;

    blocks.insert(block_info.id().as_ref(), block)?;

    info.insert(block_info.id().as_ref(), block_info.serialize().as_slice())?;

    Ok(Ok(()))
}

#[inline]
fn put_tag_impl(
    info: &TransactionalTree,
    tags: &TransactionalTree,
    cold_store_blocks: &TransactionalTree,
    tag_name: &str,
    block_hash: &[u8],
    id_size: usize,
) -> Result<Result<(), Error>, ConflictableTransactionError<()>> {
    match info.get(block_hash)? {
        Some(info_bin) => {
            let mut block_info =
                BlockInfo::deserialize(&info_bin[..], id_size, block_hash.to_vec());
            block_info.add_ref();
            let info_bin = block_info.serialize();
            info.insert(block_hash, info_bin)?;
        }
        None => {
            if cold_store_blocks.get(block_hash)?.is_none() {
                return Ok(Err(Error::BlockNotFound));
            }
        }
    }

    let maybe_old_block_hash = tags.insert(tag_name, block_hash)?;

    if let Some(old_block_hash) = maybe_old_block_hash {
        let info_bin = info.get(old_block_hash.clone())?.unwrap();
        let mut block_info =
            BlockInfo::deserialize(&info_bin[..], id_size, old_block_hash.to_vec());
        block_info.remove_ref();
        let info_bin = block_info.serialize();
        info.insert(block_info.id().as_ref(), info_bin)?;
    }

    Ok(Ok(()))
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn remove_tip_impl(
    blocks: &TransactionalTree,
    info: &TransactionalTree,
    height_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    cold_store_blocks: &TransactionalTree,
    block_id: &[u8],
    root_id: &[u8],
    id_size: usize,
) -> Result<RemoveTipResult, ConflictableTransactionError<()>> {
    // Stop when we bump into a block stored in the cold storage.
    if cold_store_blocks.get(block_id)?.is_some() {
        return Ok(RemoveTipResult::Done);
    }

    let block_info_bin = info.get(block_id)?.unwrap();
    let mut block_info_reader: &[u8] = &block_info_bin;
    let block_info = BlockInfo::deserialize(&mut block_info_reader, id_size, block_id.to_vec());

    if block_info.ref_count() != 0 {
        return Ok(RemoveTipResult::Done);
    }

    info.remove(block_id)?;
    blocks.remove(block_id)?;

    let mut height_index = block_info.chain_length().to_le_bytes().to_vec();
    height_index.extend_from_slice(block_info.id().as_ref());
    height_to_block_ids.remove(height_index)?;

    tips.remove(block_id)?;

    if block_info.parent_id().as_ref() == root_id {
        return Ok(RemoveTipResult::Done);
    }

    let maybe_parent_block_info = if cold_store_blocks.get(block_info.parent_id())?.is_some() {
        None
    } else {
        let parent_block_info_bin = info.get(block_info.parent_id())?.unwrap();
        let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
        let mut parent_block_info = BlockInfo::deserialize(
            &mut parent_block_info_reader,
            id_size,
            block_info.parent_id().clone(),
        );
        parent_block_info.remove_ref();
        info.insert(
            parent_block_info.id().as_ref(),
            parent_block_info.serialize(),
        )?;

        Some(parent_block_info)
    };

    let maybe_next_tip = match maybe_parent_block_info {
        Some(parent_block_info) => {
            if parent_block_info.ref_count() == 0 {
                // If the block is inside another branch it cannot be a tip.
                // This will also apply if this tip is tagged.
                tips.insert(block_info.parent_id().as_ref(), &[])?;
                RemoveTipResult::NextTip(block_info.parent_id().as_ref().to_vec())
            } else {
                RemoveTipResult::Done
            }
        }
        None => RemoveTipResult::HitColdStore(block_info.parent_id().as_ref().to_vec()),
    };

    Ok(maybe_next_tip)
}

#[inline]
fn convert_transaction_result<T>(
    result: Result<Result<T, Error>, TransactionError<()>>,
) -> Result<T, Error> {
    match result {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(err)) => Err(err),
        Err(TransactionError::Storage(err)) => Err(err.into()),
        Err(TransactionError::Abort(())) => unreachable!(),
    }
}

#[cfg(test)]
pub mod tests {
    use super::test_utils::{Block, BlockId};
    use super::*;
    use rand_core::{OsRng, RngCore};
    use std::{collections::HashSet, iter::FromIterator};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;
    const BLOCK_NUM_PERMANENT_TEST: usize = 1024;
    const FLUSH_TO_BLOCK: usize = 512;

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a [A]) -> &'a A {
        let s = rng.next_u32() as usize;
        // this doesn't need to be uniform
        &v[s % v.len()]
    }

    pub fn generate_chain<R: RngCore>(rng: &mut R, store: &mut BlockStore) -> Vec<Block> {
        let mut blocks = vec![];

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        blocks.push(genesis_block);

        for _ in 0..10 {
            let mut parent_block = pick_from_vector(rng, &blocks).clone();
            let r = 1 + (rng.next_u32() % 99);
            for _ in 0..r {
                let block = parent_block.make_child(None);
                let block_info = BlockInfo::new(
                    block.id.serialize_as_vec(),
                    block.parent.serialize_as_vec(),
                    block.chain_length,
                );
                store
                    .put_block(&block.serialize_as_vec(), block_info)
                    .unwrap();
                parent_block = block.clone();
                blocks.push(block);
            }
        }

        blocks
    }

    fn prepare_store() -> (tempfile::TempDir, BlockStore) {
        let file = tempfile::TempDir::new().unwrap();
        let store = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();

        (file, store)
    }

    #[test]
    fn tag_get_non_existent() {
        let (_file, mut store) = prepare_store();
        assert!(store.get_tag("tip").unwrap().is_none());
    }

    #[test]
    fn tag_non_existent_block() {
        let (_file, mut store) = prepare_store();
        match store.put_tag("tip", &BlockId(0).serialize_as_vec()) {
            Err(Error::BlockNotFound) => {}
            err => panic!(err),
        }
    }

    #[test]
    fn tag_put() {
        let mut rng = OsRng;

        let (_file, mut store) = prepare_store();
        let blocks = generate_chain(&mut rng, &mut store);

        store
            .put_tag("tip", &blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();
        assert_eq!(
            store.get_tag("tip").unwrap().unwrap(),
            blocks.last().unwrap().id.serialize_as_value()
        );
    }

    #[test]
    fn tag_overwrite() {
        let mut rng = OsRng;

        let (_file, mut store) = prepare_store();
        let blocks = generate_chain(&mut rng, &mut store);

        store
            .put_tag("tip", &blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();
        store
            .put_tag("tip", &blocks.first().unwrap().id.serialize_as_vec())
            .unwrap();
        assert_eq!(
            store.get_tag("tip").unwrap().unwrap(),
            blocks.first().unwrap().id.serialize_as_value()
        );
    }

    #[test]
    fn block_read_write() {
        let (_file, mut store) = prepare_store();
        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );

        assert!(!store
            .block_exists(genesis_block_info.id().as_ref())
            .unwrap());

        store
            .put_block(
                &genesis_block.serialize_as_vec(),
                genesis_block_info.clone(),
            )
            .unwrap();
        let genesis_block_restored = store
            .get_block_info(&genesis_block.id.serialize_as_vec())
            .unwrap();

        assert!(store
            .block_exists(genesis_block_info.id().as_ref())
            .unwrap());

        assert_eq!(
            &genesis_block.id.serialize_as_vec()[..],
            genesis_block_restored.id().as_ref()
        );
        assert_eq!(
            &genesis_block.parent.serialize_as_vec()[..],
            genesis_block_restored.parent_id().as_ref()
        );
        assert_eq!(
            genesis_block.chain_length,
            genesis_block_restored.chain_length()
        );

        assert_eq!(
            vec![genesis_block.id.serialize_as_value()],
            store.get_tips_ids().unwrap()
        );
    }

    #[test]
    pub fn nth_ancestor() {
        let mut rng = OsRng;
        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();
        let blocks = generate_chain(&mut rng, &mut store);

        let mut blocks_fetched = 0;
        let mut total_distance = 0;
        let nr_tests = 1000;

        for _ in 0..nr_tests {
            let block = pick_from_vector(&mut rng, &blocks);
            assert_eq!(
                store.get_block(&block.id.serialize_as_vec()).unwrap(),
                block.serialize_as_value()
            );

            let distance = rng.next_u32() % block.chain_length;
            total_distance += distance;

            let ancestor_info = for_path_to_nth_ancestor(
                &mut store,
                &block.id.serialize_as_vec(),
                distance,
                |_| {
                    blocks_fetched += 1;
                },
            )
            .unwrap();

            assert_eq!(ancestor_info.chain_length() + distance, block.chain_length);
        }

        let blocks_per_test = blocks_fetched as f64 / nr_tests as f64;

        println!(
            "fetched {} intermediate blocks ({} per test), total distance {}",
            blocks_fetched, blocks_per_test, total_distance
        );
    }

    #[test]
    fn simultaneous_read_write() {
        let mut rng = OsRng;
        let file = tempfile::TempDir::new().unwrap();
        let mut conn = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        conn.put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            blocks.push(block.clone());
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            conn.put_block(&block.serialize_as_vec(), block_info)
                .unwrap()
        }

        let mut conn_1 = conn.clone();
        let blocks_1 = blocks.clone();

        let thread_1 = std::thread::spawn(move || {
            for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
                let block_id = blocks_1
                    .get(rng.next_u32() as usize % blocks_1.len())
                    .unwrap()
                    .id
                    .serialize_as_vec();
                conn_1.get_block(&block_id).unwrap();
            }
        });

        let thread_2 = std::thread::spawn(move || {
            for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
                let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
                let block = last_block.make_child(None);
                let block_info = BlockInfo::new(
                    block.id.serialize_as_vec(),
                    block.parent.serialize_as_vec(),
                    block.chain_length,
                );
                conn.put_block(&block.serialize_as_vec(), block_info)
                    .unwrap()
            }
        });

        thread_1.join().unwrap();
        thread_2.join().unwrap();
    }

    #[test]
    fn branch_pruning() {
        const MAIN_BRANCH_LEN: usize = 100;
        const SECOND_BRANCH_LEN: usize = 25;
        const BIFURCATION_POINT: usize = 50;

        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();

        let mut main_branch_blocks = vec![];

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();

        let mut block = genesis_block.make_child(None);

        main_branch_blocks.push(genesis_block);

        for _i in 1..MAIN_BRANCH_LEN {
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            main_branch_blocks.push(block.clone());
            block = block.make_child(None);
        }

        let mut second_branch_blocks = vec![main_branch_blocks[BIFURCATION_POINT].clone()];

        block = main_branch_blocks[BIFURCATION_POINT].make_child(None);

        for _i in 1..SECOND_BRANCH_LEN {
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            second_branch_blocks.push(block.clone());
            block = block.make_child(None);
        }

        let expected_tips = {
            let mut hs = HashSet::new();
            hs.insert(main_branch_blocks.last().unwrap().id.serialize_as_value());
            hs.insert(second_branch_blocks.last().unwrap().id.serialize_as_value());
            hs
        };
        let actual_tips = HashSet::from_iter(store.get_tips_ids().unwrap().into_iter());
        assert_eq!(expected_tips, actual_tips);

        store
            .prune_branch(&second_branch_blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();

        assert_eq!(
            vec![main_branch_blocks.last().unwrap().id.serialize_as_value()],
            store.get_tips_ids().unwrap()
        );

        store
            .get_block(&second_branch_blocks[0].id.serialize_as_vec())
            .unwrap();

        for i in 1..SECOND_BRANCH_LEN {
            let block_result = store.get_block(&second_branch_blocks[i].id.serialize_as_vec());
            assert!(matches!(block_result, Err(Error::BlockNotFound)));
        }

        // tagged branch must not be removed
        store
            .put_tag(
                "tip",
                &main_branch_blocks.last().unwrap().id.serialize_as_vec(),
            )
            .unwrap();
        store
            .prune_branch(&main_branch_blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();
        assert!(store
            .block_exists(&main_branch_blocks.last().unwrap().id.serialize_as_vec())
            .unwrap());
    }

    #[test]
    fn get_blocks_by_chain_length() {
        const N_BLOCKS: usize = 5;

        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();

        let mut blocks = vec![];

        for _i in 0..N_BLOCKS {
            let block = genesis_block.make_child(None);
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            let block = block.serialize_as_vec();
            store.put_block(&block, block_info).unwrap();
            blocks.push(block);
        }

        let chain_length = genesis_block.chain_length + 1;

        let expected: HashSet<_, std::collections::hash_map::RandomState> = HashSet::from_iter(
            blocks
                .into_iter()
                .map(|block| Value::owned(block.into_boxed_slice())),
        );
        let actual = HashSet::from_iter(
            store
                .get_blocks_by_chain_length(chain_length)
                .unwrap()
                .into_iter(),
        );

        assert_eq!(expected, actual);
    }

    fn generate_two_branches() -> (tempfile::TempDir, BlockStore, Vec<Block>, Vec<Block>) {
        const MAIN_BRANCH_LEN: usize = 100;
        const SECOND_BRANCH_LEN: usize = 25;
        const BIFURCATION_POINT: usize = 50;

        let (file, mut store) = prepare_store();

        let mut main_branch_blocks = vec![];

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();

        let mut block = genesis_block.make_child(None);

        main_branch_blocks.push(genesis_block);

        for _i in 1..MAIN_BRANCH_LEN {
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            main_branch_blocks.push(block.clone());
            block = block.make_child(None);
        }

        let mut second_branch_blocks = vec![main_branch_blocks[BIFURCATION_POINT].clone()];

        block = main_branch_blocks[BIFURCATION_POINT].make_child(None);

        for _i in 1..SECOND_BRANCH_LEN {
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            second_branch_blocks.push(block.clone());
            block = block.make_child(None);
        }

        (file, store, main_branch_blocks, second_branch_blocks)
    }

    #[test]
    fn is_ancestor_same_branch() {
        const FIRST: usize = 20;
        const SECOND: usize = 30;

        let (_file, mut store, main_branch_blocks, _) = generate_two_branches();

        let result = store
            .is_ancestor(
                &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
                &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
            )
            .unwrap()
            .expect("should be a non-None result") as usize;
        assert!(SECOND - FIRST == result);
    }

    #[test]
    fn is_ancestor_wrong_order() {
        const FIRST: usize = 30;
        const SECOND: usize = 20;

        let (_file, mut store, main_branch_blocks, _) = generate_two_branches();

        let result = store
            .is_ancestor(
                &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
                &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
            )
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn is_ancestor_different_branches() {
        const FIRST: usize = 60;
        const SECOND: usize = 10;

        let (_file, mut store, main_branch_blocks, second_branch_blocks) = generate_two_branches();

        let result = store
            .is_ancestor(
                &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
                &second_branch_blocks[SECOND].id.serialize_as_vec()[..],
            )
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn is_ancestor_permanent_hot() {
        const COLD_STORAGE_START: usize = 40;
        const FIRST: usize = 10;
        const SECOND: usize = 50;

        let (_file, mut store, main_branch_blocks, _) = generate_two_branches();

        store
            .flush_to_cold_store(&main_branch_blocks[COLD_STORAGE_START].id.serialize_as_vec())
            .unwrap();

        let result = store
            .is_ancestor(
                &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
                &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
            )
            .unwrap()
            .expect("should be a non-None result") as usize;
        assert!(SECOND - FIRST == result);
    }

    #[test]
    fn is_ancestor_only_permanent() {
        const COLD_STORAGE_START: usize = 40;
        const FIRST: usize = 10;
        const SECOND: usize = 20;

        let (_file, mut store, main_branch_blocks, _) = generate_two_branches();

        store
            .flush_to_cold_store(&main_branch_blocks[COLD_STORAGE_START].id.serialize_as_vec())
            .unwrap();

        let result = store
            .is_ancestor(
                &main_branch_blocks[FIRST].id.serialize_as_vec()[..],
                &main_branch_blocks[SECOND].id.serialize_as_vec()[..],
            )
            .unwrap()
            .expect("should be a non-None result") as usize;
        assert!(SECOND - FIRST == result);
    }

    fn prepare_permament_store() -> (tempfile::TempDir, BlockStore, Vec<Block>) {
        const BLOCK_DATA_LENGTH: usize = 512;

        let mut rng = OsRng;
        let mut block_data = [0; BLOCK_DATA_LENGTH];

        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(
            file.path(),
            BlockId(0).serialize_as_vec(),
            BlockId(0).serialize_as_vec().len(),
            1,
        )
        .unwrap();

        let mut blocks = vec![];

        rng.fill_bytes(&mut block_data);
        let genesis_block = Block::genesis(Some(block_data.clone().to_vec().into_boxed_slice()));
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();

        rng.fill_bytes(&mut block_data);
        let mut block =
            genesis_block.make_child(Some(block_data.clone().to_vec().into_boxed_slice()));

        blocks.push(genesis_block);

        for _i in 1..BLOCK_NUM_PERMANENT_TEST {
            let block_info = BlockInfo::new(
                block.id.serialize_as_vec(),
                block.parent.serialize_as_vec(),
                block.chain_length,
            );
            store
                .put_block(&block.serialize_as_vec(), block_info)
                .unwrap();
            blocks.push(block.clone());
            rng.fill_bytes(&mut block_data);
            block = block.make_child(Some(block_data.clone().to_vec().into_boxed_slice()));
        }

        store
            .flush_to_cold_store(&blocks[FLUSH_TO_BLOCK].id.serialize_as_vec())
            .unwrap();

        (file, store, blocks)
    }

    #[test]
    fn permanent_store_read() {
        let (_file, mut store, blocks) = prepare_permament_store();

        for block in blocks.iter() {
            let block_id = block.id.serialize_as_vec();

            let block_info = store.get_block_info(&block_id).unwrap();
            assert_eq!(&block.id.serialize_as_vec()[..], block_info.id().as_ref());
            assert_eq!(
                &block.parent.serialize_as_vec()[..],
                block_info.parent_id().as_ref()
            );
            assert_eq!(block.chain_length, block_info.chain_length());

            let actual_block = store.get_block(&block_id).unwrap();
            assert_eq!(block.serialize_as_value().as_ref(), actual_block.as_ref());
        }
    }

    #[test]
    fn permanent_store_tag() {
        const TAGS_TEST_HEIGHT: usize = 20;

        let (_file, mut store, blocks) = prepare_permament_store();

        store
            .put_tag("test1", &blocks[TAGS_TEST_HEIGHT].id.serialize_as_vec())
            .unwrap();
    }

    #[test]
    fn permanent_store_prune_main_branch() {
        let (_file, mut store, blocks) = prepare_permament_store();

        store
            .prune_branch(&blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();

        for i in 0..=FLUSH_TO_BLOCK {
            assert!(store
                .block_exists(&blocks[i].id.serialize_as_vec())
                .unwrap());
        }

        for i in (FLUSH_TO_BLOCK + 1)..FLUSH_TO_BLOCK {
            assert!(!store
                .block_exists(&blocks[i].serialize_as_vec())
                .unwrap());
        }

        assert_eq!(
            vec![blocks[FLUSH_TO_BLOCK].id.serialize_as_value()],
            store.get_tips_ids().unwrap()
        );
    }

    #[test]
    fn permanent_store_get_by_chain_length() {
        const HEIGHT: usize = 20;

        let (_file, mut store, blocks) = prepare_permament_store();

        let chain_length = blocks[HEIGHT].chain_length;
        assert_eq!(
            vec![blocks[HEIGHT].serialize_as_value()],
            store.get_blocks_by_chain_length(chain_length).unwrap()
        );
    }
}
