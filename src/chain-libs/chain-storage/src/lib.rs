//! Block storage crate.
//!
//! # Data model
//!
//! This storage is designed to be independent from a particular block strucure.
//! At the same time, since it is designed specifically for block chains, it
//! handles the minimum amount of data required to maintain consistency. Such
//! data includes:
//!
//! * Block ID
//! * ID of the parent block
//! * Chain length
//!
//! This data is provided alongside a block in the `BlockInfo` structure.
//!
//! # Volatile + permanent storage model
//!
//! Since blockchain can be branching extensively, this library provides the
//! mechanism to have multiple blockchain branches. However, doing so for all
//! block screates a lot of overhead for maintaining volatile storage. At the
//! same time, branches usually die out pretty fast.
//!
//! Given that, the storage is separated into two parts:
//!
//! * Volatile storage should be used for the part of a blockchain where a
//!   developer may need access to different branches.
//! * Permanent storage for the part of the blockchain that is guaranteed not to
//!   change anymore.
//!
//! ## Moving blocks to the permanent store.
//!
//! If you have this block structure and call
//! `store.flush_to_permanent_store(block 2 id)`, then `Block 1` and `Block 2`
//! will be moved to the permanent storage. If you call
//! `store.flush_to_permanent_store(block 3 id)`, `Block 3` will also be moved
//! to the permanent store, but `Block 3'` will still exist in the volatile store.
//! Note that if you call `store.get_blocks_by_chain_length(3)` only `Block 3`
//! (which is in the permanent store) will be returned; and you cannot call
//! `store.flush_to_permanent_store(block 3' id)` now.
//!
//! __fig.1 - note that root does not actually exist, this is an ID referredby__
//! __the first block in the chain__
//!
//! ```ignore
//! +---------+       +---------+
//! |         |       |         |
//! | Block 4 |       | Block 4'|
//! |         |       |         |
//! +---------+       +---------+
//!      |                 |
//!      |                 |
//!      v                 v
//! +---------+       +---------+
//! |         |       |         |
//! | Block 3 |       | Block 3'|
//! |         |       |         |
//! +---------+       +---------+
//!      |                 |
//!      |                 |
//!      v                 |
//! +---------+            |
//! |         |            |
//! | Block 2 +<-----------+
//! |         |
//! +---------+
//!      |
//!      |
//!      v
//! +---------+
//! |         |
//! | Block 1 |
//! |         |
//! +---------+
//!      |
//!      |
//!      v
//! +---------+
//! |         |
//! |  (root) |
//! |         |
//! +---------+
//! ```
//!
//! ## Removing stale branches
//!
//! If you want to clean up branches, do the following:
//!
//! * Call `store.get_tips_ids()`, in our example it will return
//!   `[Block 4 id, Block 4' id]`;
//! * Determine which branches do you want to remove.
//! * Call, for example, `store.prune_branch(Block 4' id)`.
//!
//! ## Performance benefits of permanent storage
//!
//! Since blocks in the permanent storage are stored just one after another (the
//! structure is `block_length.to_le_bytes() ++ block_bytes` and a file with
//! references to blocks in the order they were added to the storage), it allows
//! for the following scenarios:
//!
//! * Very fast block iteration
//! * Transferring a portion of the data file over the network without locking
//!   it.
//! * O(1) access by chain length.
//!
//! __fig. 2 - permanent storage structure__
//!
//! ```ignore
//! store                  block no. index
//!
//! +--------------+       +-------------+
//! |              |       |             |
//! | block1       |       | block1 pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | block2       |       | block2 pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | ...          |       | ...         |
//! |              |       |             |
//! +--------------+       +-------------+
//! |              |       |             |
//! | blockn       |       | blockn pos  |
//! |              |       |             |
//! +--------------+       +-------------+
//! ```
//!
//! # Storage directory layout
//!
//! ```ignore
//! store
//! ├── permanent       - permanent storage directory
//! │   └── flatfile    - storage file that can be transferred over the network
//! └── volatile        - volatile storage
//! ```

mod block_info;
mod error;
mod iterator;
mod permanent_store;
#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils;
#[cfg(test)]
mod tests;
mod value;

use permanent_store::PermanentStore;
use sled::transaction::{
    ConflictableTransactionError, TransactionError, Transactional, TransactionalTree,
};
use std::path::Path;

pub use block_info::BlockInfo;
pub use error::{ConsistencyFailure, Error};
pub use iterator::StorageIterator;
pub use value::Value;

#[derive(Clone)]
pub struct BlockStore {
    volatile: sled::Db,
    permanent: PermanentStore,
    root_id: Value,
    id_length: usize,
}

enum RemoveTipResult {
    NextTip { id: Vec<u8> },
    HitPermanentStore { id: Vec<u8> },
    Done,
}

impl From<Error> for ConflictableTransactionError<Error> {
    fn from(from: Error) -> Self {
        ConflictableTransactionError::Abort(from)
    }
}

impl From<ConsistencyFailure> for ConflictableTransactionError<Error> {
    fn from(from: ConsistencyFailure) -> Self {
        ConflictableTransactionError::Abort(from.into())
    }
}

impl From<TransactionError<Error>> for Error {
    fn from(from: TransactionError<Error>) -> Self {
        match from {
            TransactionError::Abort(err) => err,
            TransactionError::Storage(err) => err.into(),
        }
    }
}

/// Names of trees in `sled` storage. For documentation about trees please refer
/// to `sled` docs.
mod tree {
    /// Binary data of blocks stored in the volatile storage.
    pub const BLOCKS: &str = "blocks";
    /// Correspondence between IDs and chain lengths of blocks stored in the
    /// permanent storage.
    pub const PERMANENT_STORE_BLOCKS: &str = "permanent_store";
    /// Block information (see `BlockInfo`) for volatile storage.
    pub const INFO: &str = "info";
    /// Maintains conversion from chain length to block IDs. This tree has empty
    /// values and keys in the form of `chain_length.to_le_bytes() ++ block_id`.
    /// Such structure allows to get all blocks on the given chain length by
    /// using prefix `chain_length.to_le_bytes()`. `sled` allows to iterate over
    /// key-value pairs with the same prefix.
    pub const CHAIN_LENGTH_INDEX: &str = "length_to_block_ids";
    /// Holds references to blocks in the volatile storage that have no
    /// descendants. This allows to quickly determine which branches should be
    /// removed.
    pub const BRANCHES_TIPS: &str = "branches_tips";
    /// Converts a tag name to a block ID.
    pub const TAGS: &str = "tags";
}

impl BlockStore {
    /// Create a new storage handle. The path must not exist or should be a
    /// directory. The directory will be created if it does not exist.
    ///
    /// # Arguments
    ///
    /// * `path` - a path to the storage directory.
    /// * `root_id` - the ID of the root block which the first block in this
    ///   block chain should refer to as a parent.
    pub fn file<P: AsRef<Path>, I: Into<Value>>(path: P, root_id: I) -> Result<Self, Error> {
        let root_id = root_id.into();
        let id_length = root_id.as_ref().len();

        if !path.as_ref().exists() {
            std::fs::create_dir(path.as_ref()).map_err(Error::Open)?;
        }

        let volatile_path = path.as_ref().join("volatile");
        let permanent_path = path.as_ref().join("permanent");

        let volatile = sled::open(volatile_path)?;

        let block_id_index = volatile.open_tree(tree::PERMANENT_STORE_BLOCKS)?;
        let permanent = PermanentStore::file(permanent_path, block_id_index, root_id.clone())?;

        Ok(Self {
            volatile,
            permanent,
            root_id,
            id_length,
        })
    }

    /// Open a temporary in-memory database.
    ///
    /// # Arguments
    ///
    /// * `root_id` - the ID of the root block which the first block in this
    ///   block chain should refer to as a parent.
    pub fn memory<I: Into<Value>>(root_id: I) -> Result<Self, Error> {
        let root_id = root_id.into();
        let id_length = root_id.as_ref().len();
        let volatile = sled::Config::new()
            .temporary(true)
            .open()
            .map_err(|err| Error::Open(err.into()))?;
        let block_id_index = volatile.open_tree(tree::PERMANENT_STORE_BLOCKS)?;
        let permanent = PermanentStore::memory(block_id_index, root_id.clone())?;

        Ok(Self {
            volatile,
            permanent,
            root_id,
            id_length,
        })
    }

    /// Write a block to the store. The parent of the block must exist (unless
    /// it's the root id).
    ///
    /// # Arguments
    ///
    /// * `block` - a serialized representation of a block.
    /// * `block_info` - block metadata for internal needs (indexing, linking
    ///   between blocks, etc)
    pub fn put_block(&self, block: &[u8], block_info: BlockInfo) -> Result<(), Error> {
        if self.block_exists(block_info.id().as_ref())? {
            return Err(Error::BlockAlreadyPresent);
        }

        let blocks = self.volatile.open_tree(tree::BLOCKS)?;
        let info = self.volatile.open_tree(tree::INFO)?;
        let chain_length_to_block_ids = self.volatile.open_tree(tree::CHAIN_LENGTH_INDEX)?;
        let tips = self.volatile.open_tree(tree::BRANCHES_TIPS)?;

        let parent_in_permanent_store = self
            .permanent
            .contains_key(block_info.parent_id().as_ref())?;

        (&blocks, &info, &chain_length_to_block_ids, &tips)
            .transaction(|(blocks, info, chain_length_to_block_ids, tips)| {
                put_block_impl(
                    blocks,
                    info,
                    chain_length_to_block_ids,
                    tips,
                    block,
                    &block_info,
                    self.root_id.as_ref(),
                    self.id_length,
                    parent_in_permanent_store,
                )
            })
            .map_err(Into::into)
    }

    /// Get a block from the storage.
    ///
    /// # Arguments
    ///
    /// * `block_id` - the serialized block identifier.
    pub fn get_block(&self, block_id: &[u8]) -> Result<Value, Error> {
        let blocks = self.volatile.open_tree(tree::BLOCKS)?;

        if let Some(block) = self.permanent.get_block(block_id)? {
            return Ok(block);
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
    pub fn get_block_info(&self, block_id: &[u8]) -> Result<BlockInfo, Error> {
        if let Some(block_info) = self.permanent.get_block_info(block_id)? {
            return Ok(block_info);
        }

        let info = self.volatile.open_tree(tree::INFO)?;

        info.get(block_id)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .and_then(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader, self.id_length, block_id.to_vec())
            })
    }

    /// Get multiple serialized blocks from the given chain length. This will
    /// return block contents, not their IDs. If there is a block at the given
    /// chain length in the permanent storage, only this block is returned.
    /// Other branches are considered to be ready of removal if there are any.
    pub fn get_blocks_by_chain_length(&self, chain_length: u32) -> Result<Vec<Value>, Error> {
        if let Some(block) = self.permanent.get_block_by_chain_length(chain_length) {
            return Ok(vec![block]);
        }

        let blocks = self.volatile.open_tree(tree::BLOCKS)?;
        let chain_length_to_block_ids = self.volatile.open_tree(tree::CHAIN_LENGTH_INDEX)?;

        let chain_length_index_prefix = chain_length.to_le_bytes();
        chain_length_to_block_ids
            .scan_prefix(chain_length_index_prefix)
            .map(|scan_result| {
                let block_id = scan_result.map(|(key, _)| Vec::from(&key[4..key.len()]))?;

                blocks
                    .get(block_id)?
                    .ok_or(Error::Inconsistent(ConsistencyFailure::ChainLength))
                    .map(Value::volatile)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Add a tag for a given block. The block id can be later retrieved by this
    /// tag.
    pub fn put_tag(&self, tag_name: &str, block_id: &[u8]) -> Result<(), Error> {
        let info = self.volatile.open_tree(tree::INFO)?;
        let tags = self.volatile.open_tree(tree::TAGS)?;
        let permanent_store_index = self.permanent.block_id_index();

        (&info, &tags, permanent_store_index)
            .transaction(move |(info, tags, permanent_store_index)| {
                put_tag_impl(
                    info,
                    tags,
                    permanent_store_index,
                    tag_name,
                    block_id,
                    self.id_length,
                )
            })
            .map_err(Into::into)
    }

    /// Get the block ID for the given tag.
    pub fn get_tag(&self, tag_name: &str) -> Result<Option<Value>, Error> {
        let tags = self.volatile.open_tree(tree::TAGS)?;

        tags.get(tag_name)
            .map(|maybe_id_bin| maybe_id_bin.map(Value::volatile))
            .map_err(Into::into)
    }

    /// Get identifier of all branches tips.
    pub fn get_tips_ids(&self) -> Result<Vec<Value>, Error> {
        let tips = self.volatile.open_tree(tree::BRANCHES_TIPS)?;

        tips.iter()
            .map(|id_result| id_result.map(|(id, _)| Value::volatile(id)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Prune a branch with the given tip id from the storage.
    pub fn prune_branch(&self, tip_id: &[u8]) -> Result<(), Error> {
        let tips = self.volatile.open_tree(tree::BRANCHES_TIPS)?;

        if !tips.contains_key(tip_id)? {
            return Err(Error::BranchNotFound);
        }

        let blocks = self.volatile.open_tree(tree::BLOCKS)?;
        let info = self.volatile.open_tree(tree::INFO)?;
        let chain_length_to_block_ids = self.volatile.open_tree(tree::CHAIN_LENGTH_INDEX)?;
        let permanent_store_index = self.permanent.block_id_index();

        let result = (
            &blocks,
            &info,
            &chain_length_to_block_ids,
            &tips,
            permanent_store_index,
        )
            .transaction(
                |(blocks, info, chain_length_to_block_ids, tips, permanent_store_index)| {
                    let mut result = RemoveTipResult::NextTip {
                        id: Vec::from(tip_id),
                    };

                    while let RemoveTipResult::NextTip { id } = &result {
                        result = remove_tip_impl(
                            blocks,
                            info,
                            chain_length_to_block_ids,
                            tips,
                            permanent_store_index,
                            id,
                            self.root_id.as_ref(),
                            self.id_length,
                        )?;
                    }

                    Ok(result)
                },
            )?;

        if let RemoveTipResult::HitPermanentStore { id } = result {
            let block_info = self.get_block_info(&id).map_err(|err| match err {
                Error::BlockNotFound => ConsistencyFailure::MissingPermanentBlock.into(),
                err => err,
            })?;
            let chain_length = block_info.chain_length() + 1;

            if self.get_blocks_by_chain_length(chain_length)?.is_empty() {
                tips.insert(block_info.id(), &[])?;
            }
        }

        Ok(())
    }

    /// Check if the block with the given id exists.
    pub fn block_exists(&self, block_id: &[u8]) -> Result<bool, Error> {
        let info = self.volatile.open_tree(tree::INFO)?;

        if self.permanent.contains_key(block_id)? {
            return Ok(true);
        }

        info.get(block_id)
            .map(|maybe_block| maybe_block.is_some())
            .map_err(Into::into)
    }

    /// Determine whether block identified by `ancestor_id` is an ancestor of
    /// block identified by `descendent_id`.
    ///
    /// Returned values:
    /// * `Ok(Some(dist))` - `ancestor` is ancestor of `descendent` and there
    ///   are `dist` blocks between them
    /// * `Ok(None)` - `ancestor` is not ancestor of `descendent`
    /// * `Err(error)` - `ancestor` or `descendent` was not found
    pub fn is_ancestor(
        &self,
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

    /// Get n-th (n = `distance`) ancestor of the block, identified by
    /// `block_id`.
    pub fn get_nth_ancestor(&self, block_id: &[u8], distance: u32) -> Result<BlockInfo, Error> {
        for_path_to_nth_ancestor(self, block_id, distance, |_| {})
    }

    /// Move all blocks up to the provided block ID to the permanent block
    /// storage.
    pub fn flush_to_permanent_store(&self, to_block: &[u8]) -> Result<(), Error> {
        let mut block_infos = Vec::new();

        let mut current_block_id = to_block;

        while let Some(block_info) =
            self.get_block_info(current_block_id)
                .map(Some)
                .or_else(|err| match err {
                    Error::BlockNotFound => Ok(None),
                    e => Err(e),
                })?
        {
            block_infos.push(block_info);
            current_block_id = block_infos.last().unwrap().parent_id().as_ref();
        }

        if block_infos.is_empty() {
            return Ok(());
        }

        let blocks = block_infos
            .iter()
            .rev()
            .map(|block_info| self.get_block(block_info.id().as_ref()))
            .collect::<Result<Vec<_>, Error>>()?;
        let block_refs: Vec<_> = blocks.iter().map(|block| block.as_ref()).collect();
        let ids: Vec<_> = block_infos
            .iter()
            .rev()
            .map(|block_info| block_info.id().as_ref())
            .collect();
        // this `unwrap` will never fail because `block_infos` cannot be empty at this point
        let start_chain_length = block_infos.last().unwrap().chain_length();
        self.permanent
            .put_blocks(start_chain_length, &ids, &block_refs)?;

        let blocks = self.volatile.open_tree(tree::BLOCKS)?;
        let info = self.volatile.open_tree(tree::INFO)?;

        for (i, block_info) in block_infos.iter().enumerate() {
            let key = block_info.id().as_ref();
            let chain_length = start_chain_length + i as u32;

            let mut chain_length_index = chain_length.to_le_bytes().to_vec();
            chain_length_index.extend_from_slice(key);

            info.remove(key)?;
            blocks.remove(key)?;
        }

        Ok(())
    }

    /// Iterate to the given block starting from the block at the given
    /// `distance - 1`. `distance == 1` means that only `to_block` will be
    /// iterated. `distance == 0` means empty iterator.
    pub fn iter(
        &self,
        to_block: &[u8],
        distance: u32,
    ) -> Result<impl Iterator<Item = Result<Value, Error>>, Error> {
        let block_info = self.volatile.open_tree(tree::INFO)?;
        let blocks = self.volatile.open_tree(tree::BLOCKS)?;
        StorageIterator::new(
            Value::from(to_block.to_vec()),
            distance,
            self.permanent.clone(),
            block_info,
            blocks,
        )
    }
}

/// Like `BlockStore::get_nth_ancestor`, but calls the closure `callback` with
/// each intermediate block encountered while travelling from `block_id` to
/// its n-th ancestor.
///
pub fn for_path_to_nth_ancestor<F>(
    store: &BlockStore,
    block_id: &[u8],
    distance: u32,
    mut callback: F,
) -> Result<BlockInfo, Error>
where
    F: FnMut(&BlockInfo),
{
    let mut current = store.get_block_info(block_id)?;

    if distance > current.chain_length() {
        return Err(Error::CannotIterate);
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
    info: &TransactionalTree,
    chain_length_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    block: &[u8],
    block_info: &BlockInfo,
    root_id: &[u8],
    id_length: usize,
    parent_external: bool,
) -> Result<(), ConflictableTransactionError<Error>> {
    let parent_in_volatile_store = if parent_external || block_info.parent_id().as_ref() == root_id
    {
        false
    } else if info.get(block_info.parent_id())?.is_none() {
        return Err(Error::MissingParent.into());
    } else {
        true
    };

    if parent_in_volatile_store {
        let parent_block_info_bin = info
            .get(block_info.parent_id())?
            .ok_or(ConsistencyFailure::BlockInfo)?;
        let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
        let mut parent_block_info = BlockInfo::deserialize(
            &mut parent_block_info_reader,
            id_length,
            block_info.parent_id().clone(),
        )?;
        parent_block_info.add_parent_ref();
        info.insert(
            parent_block_info.id().as_ref(),
            parent_block_info.serialize()?,
        )?;
    }

    tips.remove(block_info.parent_id().as_ref())?;
    tips.insert(block_info.id().as_ref(), &[])?;

    let mut chain_length_index = block_info.chain_length().to_le_bytes().to_vec();
    chain_length_index.extend_from_slice(block_info.id().as_ref());
    chain_length_to_block_ids.insert(chain_length_index, &[])?;

    blocks.insert(block_info.id().as_ref(), block)?;

    info.insert(block_info.id().as_ref(), block_info.serialize()?)?;

    Ok(())
}

#[inline]
fn put_tag_impl(
    info: &TransactionalTree,
    tags: &TransactionalTree,
    permanent_store_index: &TransactionalTree,
    tag_name: &str,
    block_id: &[u8],
    id_size: usize,
) -> Result<(), ConflictableTransactionError<Error>> {
    if let Some(info_bin) = info.get(block_id)? {
        let mut block_info = BlockInfo::deserialize(&info_bin[..], id_size, block_id.to_vec())?;
        block_info.add_tag_ref();
        let info_bin = block_info.serialize()?;
        info.insert(block_id, info_bin)?;
    } else if !permanent_store_index
        .get(block_id)
        .map(|maybe_block| maybe_block.is_some())?
    {
        return Err(Error::BlockNotFound.into());
    }

    let maybe_old_block_id = tags.insert(tag_name, block_id)?;

    if let Some(old_block_id) = maybe_old_block_id {
        let info_bin = info
            .get(old_block_id.clone())?
            .ok_or(ConsistencyFailure::TaggedBlock)?;
        let mut block_info = BlockInfo::deserialize(&info_bin[..], id_size, old_block_id.to_vec())?;
        block_info.remove_tag_ref();
        let info_bin = block_info.serialize()?;
        info.insert(block_info.id().as_ref(), info_bin)?;
    }

    Ok(())
}

#[inline]
#[allow(clippy::too_many_arguments)]
fn remove_tip_impl(
    blocks: &TransactionalTree,
    info: &TransactionalTree,
    chain_length_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    permanent_store_index: &TransactionalTree,
    block_id: &[u8],
    root_id: &[u8],
    id_size: usize,
) -> Result<RemoveTipResult, ConflictableTransactionError<Error>> {
    // Stop when we bump into a block stored in the permanent storage.
    if permanent_store_index
        .get(block_id)
        .map(|maybe_block| maybe_block.is_some())?
    {
        return Ok(RemoveTipResult::Done);
    }

    let block_info_bin = info.get(block_id)?.ok_or(ConsistencyFailure::BlockInfo)?;
    let mut block_info_reader: &[u8] = &block_info_bin;
    let block_info = BlockInfo::deserialize(&mut block_info_reader, id_size, block_id.to_vec())?;

    if block_info.ref_count() != 0 {
        return Ok(RemoveTipResult::Done);
    }

    info.remove(block_id)?;
    blocks.remove(block_id)?;

    let mut chain_length_index = block_info.chain_length().to_le_bytes().to_vec();
    chain_length_index.extend_from_slice(block_info.id().as_ref());
    chain_length_to_block_ids.remove(chain_length_index)?;

    tips.remove(block_id)?;

    if block_info.parent_id().as_ref() == root_id {
        return Ok(RemoveTipResult::Done);
    }

    let parent_permanent = permanent_store_index
        .get(block_info.parent_id().as_ref())
        .map(|maybe_block| maybe_block.is_some())?;

    if parent_permanent {
        return Ok(RemoveTipResult::HitPermanentStore {
            id: block_info.parent_id().as_ref().to_vec(),
        });
    }

    let parent_block_info_bin = info
        .get(block_info.parent_id())?
        .ok_or(ConsistencyFailure::MissingParentBlock)?;
    let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
    let mut parent_block_info = BlockInfo::deserialize(
        &mut parent_block_info_reader,
        id_size,
        block_info.parent_id().clone(),
    )?;
    parent_block_info.remove_parent_ref();
    info.insert(
        parent_block_info.id().as_ref(),
        parent_block_info.serialize()?,
    )?;

    // If the block is inside another branch it cannot be a tip.
    if parent_block_info.parent_ref_count() != 0 {
        return Ok(RemoveTipResult::Done);
    }

    tips.insert(block_info.parent_id().as_ref(), &[])?;

    // A referenced block cannot be removed.
    if parent_block_info.ref_count() != 0 {
        return Ok(RemoveTipResult::Done);
    }

    Ok(RemoveTipResult::NextTip {
        id: block_info.parent_id().as_ref().to_vec(),
    })
}
