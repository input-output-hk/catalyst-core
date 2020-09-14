use crate::{
    permanent_store::PermanentStore, BlockInfo, ConsistencyFailure, Error, StorageIterator, Value,
};
use sled::{
    transaction::{
        ConflictableTransactionError, TransactionError, Transactional, TransactionalTree,
    },
    Tree,
};
use std::path::Path;

#[derive(Clone)]
pub struct BlockStore {
    permanent: PermanentStore,
    root_id: Value,
    id_length: usize,

    blocks_tree: Tree,
    info_tree: Tree,
    chain_length_index_tree: Tree,
    branches_tips_tree: Tree,
    tags_tree: Tree,

    // needs to be kept so that the database is always closed correctly
    _db: sled::Db,
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

// Names of trees in `sled` storage. For documentation about trees please refer
// to `sled` docs.
mod tree {
    // Binary data of blocks stored in the volatile storage.
    pub const BLOCKS: &str = "blocks";
    // Correspondence between IDs and chain lengths of blocks stored in the
    // permanent storage.
    pub const PERMANENT_STORE_BLOCKS: &str = "permanent_store";
    // Block information (see `BlockInfo`) for volatile storage.
    pub const INFO: &str = "info";
    // Maintains conversion from chain length to block IDs. This tree has empty
    // values and keys in the form of `bytes(chain_length) ++ block_id`. Such
    // structure allows to get all blocks on the given chain length by using
    // prefix `bytes(chain_length)`. `sled` allows to iterate over key-value
    // pairs with the same prefix.
    pub const CHAIN_LENGTH_INDEX: &str = "length_to_block_ids";
    // Holds references to blocks in the volatile storage that have no
    // descendants. This allows to quickly determine which branches should be
    // removed.
    pub const BRANCHES_TIPS: &str = "branches_tips";
    // Converts a tag name to a block ID.
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
    pub fn file<P: AsRef<Path>, I: Into<Value> + Clone>(
        path: P,
        root_id: I,
    ) -> Result<Self, Error> {
        if !path.as_ref().exists() {
            std::fs::create_dir(path.as_ref()).map_err(Error::Open)?;
        }

        let volatile_path = path.as_ref().join("volatile");
        let permanent_path = path.as_ref().join("permanent");

        let volatile = sled::open(volatile_path)?;

        let block_id_index = volatile.open_tree(tree::PERMANENT_STORE_BLOCKS)?;
        let permanent = PermanentStore::file(permanent_path, block_id_index, root_id.clone())?;

        Self::new(root_id, volatile, permanent)
    }

    /// Open a temporary in-memory database.
    ///
    /// # Arguments
    ///
    /// * `root_id` - the ID of the root block which the first block in this
    ///   block chain should refer to as a parent.
    pub fn memory<I: Into<Value> + Clone>(root_id: I) -> Result<Self, Error> {
        let volatile = sled::Config::new()
            .temporary(true)
            .open()
            .map_err(|err| Error::Open(err.into()))?;
        let block_id_index = volatile.open_tree(tree::PERMANENT_STORE_BLOCKS)?;
        let permanent = PermanentStore::memory(block_id_index, root_id.clone())?;

        Self::new(root_id, volatile, permanent)
    }

    fn new<I: Into<Value>>(
        root_id: I,
        volatile: sled::Db,
        permanent: PermanentStore,
    ) -> Result<Self, Error> {
        let root_id = root_id.into();
        let id_length = root_id.as_ref().len();

        let blocks_tree = volatile.open_tree(tree::BLOCKS)?;
        let info_tree = volatile.open_tree(tree::INFO)?;
        let chain_length_index_tree = volatile.open_tree(tree::CHAIN_LENGTH_INDEX)?;
        let branches_tips_tree = volatile.open_tree(tree::BRANCHES_TIPS)?;
        let tags_tree = volatile.open_tree(tree::TAGS)?;

        Ok(Self {
            permanent,
            root_id,
            id_length,

            blocks_tree,
            info_tree,
            chain_length_index_tree,
            branches_tips_tree,
            tags_tree,

            _db: volatile,
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

        let parent_in_permanent_store = self
            .permanent
            .contains_key(block_info.parent_id().as_ref())?;

        (
            &self.blocks_tree,
            &self.info_tree,
            &self.chain_length_index_tree,
            &self.branches_tips_tree,
        )
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
        if let Some(block) = self.permanent.get_block(block_id)? {
            return Ok(block);
        }

        self.blocks_tree
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

        self.get_block_info_volatile(block_id)
    }

    fn get_block_info_volatile(&self, block_id: &[u8]) -> Result<BlockInfo, Error> {
        self.info_tree
            .get(block_id)
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

        self.chain_length_index_tree
            .scan_prefix(build_chain_length_index_prefix(chain_length))
            .map(|scan_result| {
                let (block_id, _) = scan_result?;

                self.blocks_tree
                    .get(block_id_from_chain_length_index(&block_id))?
                    .ok_or(Error::Inconsistent(ConsistencyFailure::ChainLength))
                    .map(Value::volatile)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Add a tag for a given block. The block id can be later retrieved by this
    /// tag.
    pub fn put_tag(&self, tag_name: &str, block_id: &[u8]) -> Result<(), Error> {
        let permanent_store_index = self.permanent.block_id_index();

        (&self.info_tree, &self.tags_tree, permanent_store_index)
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
        self.tags_tree
            .get(tag_name)
            .map(|maybe_id_bin| maybe_id_bin.map(Value::volatile))
            .map_err(Into::into)
    }

    /// Get identifier of all branches tips.
    pub fn get_tips_ids(&self) -> Result<Vec<Value>, Error> {
        self.branches_tips_tree
            .iter()
            .map(|id_result| id_result.map(|(id, _)| Value::volatile(id)))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Prune a branch with the given tip id from the storage.
    pub fn prune_branch(&self, tip_id: &[u8]) -> Result<(), Error> {
        if !self.branches_tips_tree.contains_key(tip_id)? {
            return Err(Error::BranchNotFound);
        }

        let permanent_store_index = self.permanent.block_id_index();

        let result = (
            &self.blocks_tree,
            &self.info_tree,
            &self.chain_length_index_tree,
            &self.branches_tips_tree,
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
                self.branches_tips_tree.insert(block_info.id(), &[])?;
            }
        }

        Ok(())
    }

    /// Check if the block with the given id exists.
    pub fn block_exists(&self, block_id: &[u8]) -> Result<bool, Error> {
        if self.permanent.contains_key(block_id)? {
            return Ok(true);
        }

        self.info_tree
            .get(block_id)
            .map(|maybe_block| maybe_block.is_some())
            .map_err(Into::into)
    }

    /// Determine whether block identified by `ancestor_id` is an ancestor of
    /// block identified by `descendant_id`.
    ///
    /// Returned values:
    /// * `Ok(Some(dist))` - `ancestor` is ancestor of `descendant` and there
    ///   are `dist` blocks between them
    /// * `Ok(None)` - `ancestor` is not ancestor of `descendant`
    /// * `Err(error)` - `ancestor` or `descendant` was not found
    pub fn is_ancestor(
        &self,
        ancestor_id: &[u8],
        descendant_id: &[u8],
    ) -> Result<Option<u32>, Error> {
        let descendant = self.get_block_info(descendant_id)?;

        if ancestor_id == descendant_id {
            return Ok(Some(0));
        }

        if ancestor_id == self.root_id.as_ref() {
            return Ok(Some(descendant.chain_length()));
        }

        // if target is in the permanent storage we only need to check chain length
        if let Some(ancestor) = self.permanent.get_block_info(ancestor_id)? {
            if ancestor.chain_length() < descendant.chain_length() {
                return Ok(Some(descendant.chain_length() - ancestor.chain_length()));
            }
            return Ok(None);
        }

        let ancestor = self.get_block_info_volatile(ancestor_id)?;

        if ancestor.chain_length() >= descendant.chain_length() {
            return Ok(None);
        }

        if descendant.parent_id() == ancestor.id() {
            return Ok(Some(1));
        }

        let mut chain_length_iter = self
            .chain_length_index_tree
            .scan_prefix(build_chain_length_index_prefix(ancestor.chain_length()));

        // if the target length is in the volatile storage and there is only one
        // block at the given length, this block is an ancestor
        if let Some(chain_length_res) = chain_length_iter.next() {
            let _ = chain_length_res?;
            if chain_length_iter.next().is_none() {
                return Ok(Some(descendant.chain_length() - ancestor.chain_length()));
            }
        }

        let mut current_block_info = descendant;
        let mut distance = 0;

        while let Some(parent_block_info) = self
            .get_block_info_volatile(current_block_info.parent_id().as_ref())
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
        let mut current = self.get_block_info(block_id)?;

        if distance > current.chain_length() {
            return Err(Error::CannotIterate);
        }

        let target = current.chain_length() - distance;

        // if target is in the permanent storage it is always an ancestor
        if let Some(info) = self.permanent.get_block_info_by_chain_length(target)? {
            return Ok(info);
        }

        let mut chain_length_iter = self
            .chain_length_index_tree
            .scan_prefix(build_chain_length_index_prefix(target));

        // if the target length is in the volatile storage and there is only one
        // block at the given length, it is an ancestor
        if let Some(chain_length_res) = chain_length_iter.next() {
            let (chain_length_index_entry, _) = chain_length_res?;
            if chain_length_iter.next().is_none() {
                return self
                    .get_block_info(block_id_from_chain_length_index(&chain_length_index_entry));
            }
        }

        // otherwise just iterate until we find the required ancestor
        while target < current.chain_length() {
            current = self.get_block_info_volatile(current.parent_id().as_ref())?;
        }

        Ok(current)
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

        for (i, block_info) in block_infos.iter().enumerate() {
            let key = block_info.id().as_ref();
            let chain_length = start_chain_length + i as u32;

            self.info_tree.remove(key)?;
            self.blocks_tree.remove(key)?;
            self.chain_length_index_tree
                .remove(build_chain_length_index(chain_length, key))?;
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
        StorageIterator::new(
            Value::from(to_block.to_vec()),
            distance,
            self.permanent.clone(),
            self.info_tree.clone(),
            self.blocks_tree.clone(),
        )
    }
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

    chain_length_to_block_ids.insert(
        build_chain_length_index(block_info.chain_length(), block_info.id().as_ref()),
        &[],
    )?;

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

    chain_length_to_block_ids.remove(build_chain_length_index(
        block_info.chain_length(),
        block_info.id().as_ref(),
    ))?;

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

#[inline]
fn build_chain_length_index_prefix(chain_length: u32) -> Vec<u8> {
    chain_length.to_be_bytes().to_vec()
}

#[inline]
fn build_chain_length_index(chain_length: u32, block_id: &[u8]) -> Vec<u8> {
    let mut chain_length_index = build_chain_length_index_prefix(chain_length);
    chain_length_index.extend_from_slice(block_id.as_ref());
    chain_length_index
}

#[inline]
fn block_id_from_chain_length_index(index: &[u8]) -> &[u8] {
    &index[std::mem::size_of::<u32>()..]
}
