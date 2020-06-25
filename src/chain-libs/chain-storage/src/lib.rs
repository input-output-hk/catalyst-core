mod block_info;
#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils;

use sled::{ConflictableTransactionError, TransactionError, Transactional, TransactionalTree};
use std::path::Path;
use thiserror::Error;

pub use block_info::BlockInfo;

#[derive(Debug, Error)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,
    #[error("database backend error")]
    BackendError(#[from] sled::Error),
    #[error("Block already present in DB")]
    BlockAlreadyPresent,
    #[error("the parent block is missing for the required write")]
    MissingParent,
    #[error("branch with the requested tip does not exist")]
    BranchNotFound,
}

#[derive(Clone)]
pub struct BlockStore {
    inner: sled::Db,
}

mod tree {
    pub const BLOCKS: &str = "blocks";
    pub const INFO: &str = "info";
    pub const CHAIN_HEIGHT_INDEX: &str = "height_to_block_ids";
    pub const BRANCHES_TIPS: &str = "branches_tips";
    pub const TAGS: &str = "tags";
}

impl BlockStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = sled::open(path)?;
        Ok(Self { inner })
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
        let blocks = self.inner.open_tree(tree::BLOCKS)?;
        let info = self.inner.open_tree(tree::INFO)?;
        let height_to_block_ids = self.inner.open_tree(tree::CHAIN_HEIGHT_INDEX)?;
        let tips = self.inner.open_tree(tree::BRANCHES_TIPS)?;

        let result = (&blocks, &info, &height_to_block_ids, &tips).transaction(
            |(blocks, info, height_to_block_ids, tips)| {
                put_block_impl(blocks, info, height_to_block_ids, tips, block, &block_info)
            },
        );

        convert_transaction_result(result)
    }

    /// Get a block from the storage.
    ///
    /// # Arguments
    ///
    /// * `block_id` - the serialized block identifier.
    pub fn get_block(&mut self, block_id: &[u8]) -> Result<Vec<u8>, Error> {
        let blocks = self.inner.open_tree(tree::BLOCKS)?;

        blocks
            .get(block_id)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| {
                let mut v = Vec::new();
                v.extend_from_slice(&block_bin);
                v
            })
    }

    /// Get the `BlockInfo` instance for the requested block.
    ///
    /// # Arguments
    ///
    /// * `block_id` - the serialized block identifier.
    pub fn get_block_info(&mut self, block_hash: &[u8]) -> Result<BlockInfo, Error> {
        let info = self.inner.open_tree(tree::INFO)?;

        info.get(block_hash)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })
    }

    /// Get multiple serialized blocks from the given height.
    pub fn get_blocks_by_chain_length(&mut self, chain_length: u32) -> Result<Vec<Vec<u8>>, Error> {
        let blocks = self.inner.open_tree(tree::BLOCKS)?;
        let height_to_block_ids = self.inner.open_tree(tree::CHAIN_HEIGHT_INDEX)?;

        let height_index_prefix = chain_length.to_le_bytes();
        height_to_block_ids
            .scan_prefix(height_index_prefix)
            .map(|scan_result| {
                let block_hash = scan_result.map(|(key, _)| Vec::from(&key[4..key.len()]))?;

                blocks
                    .get(block_hash)
                    .map(|maybe_raw_block| maybe_raw_block.unwrap().to_vec())
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Add a tag for a given block. The block id can be later retrieved by this
    /// tag.
    pub fn put_tag(&mut self, tag_name: &str, block_hash: &[u8]) -> Result<(), Error> {
        let info = self.inner.open_tree(tree::INFO)?;
        let tags = self.inner.open_tree(tree::TAGS)?;

        let result = (&info, &tags).transaction(move |(info, tags)| {
            match info.get(block_hash)? {
                Some(info_bin) => {
                    let mut block_info = BlockInfo::deserialize(&info_bin[..]);
                    block_info.add_ref();
                    let info_bin = block_info.serialize();
                    info.insert(block_hash, info_bin)?;

                    let maybe_old_block_hash = tags.insert(tag_name, block_hash)?;

                    if let Some(old_block_hash) = maybe_old_block_hash {
                        let info_bin = info.get(old_block_hash)?.unwrap();
                        let mut block_info = BlockInfo::deserialize(&info_bin[..]);
                        block_info.remove_ref();
                        let info_bin = block_info.serialize();
                        info.insert(block_info.id(), info_bin)?;
                    }
                }
                None => return Ok(Err(Error::BlockNotFound)),
            }

            Ok(Ok(()))
        });

        convert_transaction_result(result)
    }

    /// Get the block id for the given tag.
    pub fn get_tag(&mut self, tag_name: &str) -> Result<Option<Vec<u8>>, Error> {
        let tags = self.inner.open_tree(tree::TAGS)?;

        tags.get(tag_name)
            .map(|maybe_id_bin| {
                maybe_id_bin.map(|id_bin| {
                    let mut v = Vec::new();
                    v.extend_from_slice(&id_bin);
                    v
                })
            })
            .map_err(Into::into)
    }

    /// Get identifier of all branches tips.
    pub fn get_tips_ids(&mut self) -> Result<Vec<Vec<u8>>, Error> {
        let tips = self.inner.open_tree(tree::BRANCHES_TIPS)?;

        tips.iter()
            .map(|id_result| id_result.map(|(id, _)| id.to_vec()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    /// Prune a branch with the given tip id from the storage.
    pub fn prune_branch(&mut self, tip_id: &[u8]) -> Result<(), Error> {
        let tips = self.inner.open_tree(tree::BRANCHES_TIPS)?;

        if !tips.contains_key(tip_id)? {
            return Err(Error::BranchNotFound);
        }

        let blocks = self.inner.open_tree(tree::BLOCKS)?;
        let info = self.inner.open_tree(tree::INFO)?;
        let height_to_block_ids = self.inner.open_tree(tree::CHAIN_HEIGHT_INDEX)?;

        let mut maybe_current_tip = Some(Vec::from(tip_id));

        while let Some(current_tip) = &maybe_current_tip {
            let result = (&blocks, &info, &height_to_block_ids, &tips).transaction(
                |(blocks, info, height_to_block_ids, tips)| {
                    remove_tip_impl(blocks, info, height_to_block_ids, tips, current_tip)
                },
            );

            maybe_current_tip = convert_transaction_result(result)?;
        }

        Ok(())
    }

    /// Check if the block with the given id exists.
    pub fn block_exists(&mut self, block_hash: &[u8]) -> Result<bool, Error> {
        let info = self.inner.open_tree(tree::INFO)?;

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
        let info = self.inner.open_tree(tree::INFO)?;

        let descendent: BlockInfo = info
            .get(descendent_id)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })?;

        if ancestor_id == descendent_id {
            return Ok(Some(0));
        }

        if ancestor_id == vec![0u8; descendent_id.len()].as_slice() {
            return Ok(Some(descendent.chain_length()));
        }

        let ancestor = info
            .get(ancestor_id)
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })?;

        if ancestor.chain_length() >= descendent.chain_length() {
            return Ok(None);
        }

        if descendent.parent_id() == ancestor.id() {
            return Ok(Some(1));
        }

        let mut current_block_info = descendent;
        let mut distance = 0;

        while let Some(parent_block_info) =
            info.get(current_block_info.parent_id())?.map(|block_info| {
                let mut block_info_reader: &[u8] = &block_info;
                BlockInfo::deserialize(&mut block_info_reader)
            })
        {
            distance += 1;
            if parent_block_info.id() == ancestor_id {
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
}

/// Like `BlockStore::get_nth_ancestor`, but calls the closure 'callback' with
/// each intermediate block encountered while travelling from
/// 'block_hash' to its n'th ancestor.
///
/// The travelling algorithm uses back links to skip over parts of the chain,
/// so the callback will not be invoked for all blocks in the linear sequence.
pub fn for_path_to_nth_ancestor<F>(
    store: &mut BlockStore,
    block_hash: &[u8],
    distance: u32,
    mut callback: F,
) -> Result<BlockInfo, Error>
where
    F: FnMut(&BlockInfo),
{
    let info = store.inner.open_tree(tree::INFO)?;

    let mut current = info
        .get(block_hash)
        .map_err(Into::into)
        .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
        .map(|block_info_bin| {
            let mut block_info_reader: &[u8] = &block_info_bin;
            BlockInfo::deserialize(&mut block_info_reader)
        })?;

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
        current = info
            .get(current.parent_id())
            .map_err(Into::into)
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })?;
    }

    Ok(current)
}

fn put_block_impl(
    blocks: &TransactionalTree,
    info: &TransactionalTree,
    height_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    block: &[u8],
    block_info: &BlockInfo,
) -> Result<Result<(), Error>, ConflictableTransactionError<()>> {
    if info.get(block_info.id())?.is_some() {
        return Ok(Err(Error::BlockAlreadyPresent));
    }

    if block_info.parent_id() != vec![0; block_info.parent_id().len()].as_slice() {
        if info.get(block_info.parent_id())?.is_none() {
            return Ok(Err(Error::MissingParent));
        }

        let parent_block_info_bin = info.get(block_info.parent_id())?.unwrap();
        let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
        let mut parent_block_info = BlockInfo::deserialize(&mut parent_block_info_reader);
        parent_block_info.add_ref();
        info.insert(parent_block_info.id(), parent_block_info.serialize())?;
    }

    tips.remove(block_info.parent_id())?;
    tips.insert(block_info.id(), &[])?;

    let mut height_index = block_info.chain_length().to_le_bytes().to_vec();
    height_index.extend_from_slice(block_info.id());
    height_to_block_ids.insert(height_index, &[])?;

    blocks.insert(block_info.id(), block)?;

    info.insert(block_info.id(), block_info.serialize().as_slice())?;

    Ok(Ok(()))
}

fn remove_tip_impl(
    blocks: &TransactionalTree,
    info: &TransactionalTree,
    height_to_block_ids: &TransactionalTree,
    tips: &TransactionalTree,
    block_id: &[u8],
) -> Result<Result<Option<Vec<u8>>, Error>, ConflictableTransactionError<()>> {
    let block_info_bin = info.remove(block_id)?.unwrap();
    let mut block_info_reader: &[u8] = &block_info_bin;
    let block_info = BlockInfo::deserialize(&mut block_info_reader);

    if block_info.ref_count() != 0 {
        return Ok(Ok(None));
    }

    blocks.remove(block_id)?;

    let mut height_index = block_info.chain_length().to_le_bytes().to_vec();
    height_index.extend_from_slice(block_info.id());
    height_to_block_ids.remove(height_index)?;

    tips.remove(block_id)?;

    if block_info.parent_id() == vec![0; block_info.parent_id().len()].as_slice() {
        return Ok(Ok(None));
    }

    let parent_block_info_bin = info.get(block_info.parent_id())?.unwrap();
    let mut parent_block_info_reader: &[u8] = &parent_block_info_bin;
    let mut parent_block_info = BlockInfo::deserialize(&mut parent_block_info_reader);
    parent_block_info.remove_ref();
    info.insert(parent_block_info.id(), parent_block_info.serialize())?;

    let maybe_next_tip = if parent_block_info.ref_count() == 0 {
        // if the block is inside another branch it cannot be a tip
        tips.insert(block_info.parent_id(), &[])?;
        Some(block_info.parent_id().to_vec())
    } else {
        None
    };

    Ok(Ok(maybe_next_tip))
}

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

    #[test]
    pub fn test_put_get() {
        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(file.path()).unwrap();
        assert!(store.get_tag("tip").unwrap().is_none());

        match store.put_tag("tip", &BlockId(0).serialize_as_vec()) {
            Err(Error::BlockNotFound) => {}
            err => panic!(err),
        }

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo::new(
            genesis_block.id.serialize_as_vec(),
            genesis_block.parent.serialize_as_vec(),
            genesis_block.chain_length,
        );
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        let genesis_block_restored = store
            .get_block_info(&genesis_block.id.serialize_as_vec())
            .unwrap();

        assert_eq!(
            &genesis_block.id.serialize_as_vec()[..],
            genesis_block_restored.id()
        );
        assert_eq!(
            &genesis_block.parent.serialize_as_vec()[..],
            genesis_block_restored.parent_id()
        );
        assert_eq!(
            genesis_block.chain_length,
            genesis_block_restored.chain_length()
        );

        store
            .put_tag("tip", &genesis_block.id.serialize_as_vec())
            .unwrap();
        assert_eq!(
            store.get_tag("tip").unwrap().unwrap(),
            genesis_block.id.serialize_as_vec()
        );

        assert_eq!(
            vec![genesis_block.id.serialize_as_vec()],
            store.get_tips_ids().unwrap()
        );
    }

    #[test]
    pub fn test_nth_ancestor() {
        let mut rng = OsRng;
        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(file.path()).unwrap();
        let blocks = generate_chain(&mut rng, &mut store);

        let mut blocks_fetched = 0;
        let mut total_distance = 0;
        let nr_tests = 1000;

        for _ in 0..nr_tests {
            let block = pick_from_vector(&mut rng, &blocks);
            assert_eq!(
                store.get_block(&block.id.serialize_as_vec()).unwrap(),
                block.serialize_as_vec()
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
        let mut conn = BlockStore::new(file.path()).unwrap();

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
        let mut store = BlockStore::new(file.path()).unwrap();

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
            hs.insert(main_branch_blocks.last().unwrap().id.serialize_as_vec());
            hs.insert(second_branch_blocks.last().unwrap().id.serialize_as_vec());
            hs
        };
        let actual_tips = HashSet::from_iter(store.get_tips_ids().unwrap().into_iter());
        assert_eq!(expected_tips, actual_tips);

        store
            .prune_branch(&second_branch_blocks.last().unwrap().id.serialize_as_vec())
            .unwrap();

        assert_eq!(
            vec![main_branch_blocks.last().unwrap().id.serialize_as_vec()],
            store.get_tips_ids().unwrap()
        );

        store
            .get_block(&second_branch_blocks[0].id.serialize_as_vec())
            .unwrap();

        for i in 1..SECOND_BRANCH_LEN {
            let block_result = store.get_block(&second_branch_blocks[i].id.serialize_as_vec());
            assert!(matches!(block_result, Err(Error::BlockNotFound)));
        }
    }

    #[test]
    fn get_blocks_by_chain_length() {
        const N_BLOCKS: usize = 5;

        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(file.path()).unwrap();

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

        let expected: HashSet<_, std::collections::hash_map::RandomState> =
            HashSet::from_iter(blocks.into_iter());
        let actual = HashSet::from_iter(
            store
                .get_blocks_by_chain_length(chain_length)
                .unwrap()
                .into_iter(),
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn is_ancestor() {
        const MAIN_BRANCH_LEN: usize = 100;
        const SECOND_BRANCH_LEN: usize = 25;
        const BIFURCATION_POINT: usize = 50;
        const TEST_1: [usize; 2] = [20, 30];
        const TEST_2: [usize; 2] = [60, 10];

        let file = tempfile::TempDir::new().unwrap();
        let mut store = BlockStore::new(file.path()).unwrap();

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

        // same branch
        let result = store
            .is_ancestor(
                &main_branch_blocks[TEST_1[0]].id.serialize_as_vec()[..],
                &main_branch_blocks[TEST_1[1]].id.serialize_as_vec()[..],
            )
            .unwrap()
            .expect("should be a non-None result") as usize;
        assert!(TEST_1[1] - TEST_1[0] == result);

        // wrong order
        let result = store
            .is_ancestor(
                &main_branch_blocks[TEST_1[1]].id.serialize_as_vec()[..],
                &main_branch_blocks[TEST_1[0]].id.serialize_as_vec()[..],
            )
            .unwrap();
        assert!(result.is_none());

        // different branches
        let result = store
            .is_ancestor(
                &main_branch_blocks[TEST_2[0]].id.serialize_as_vec()[..],
                &second_branch_blocks[TEST_2[1]].id.serialize_as_vec()[..],
            )
            .unwrap();
        assert!(result.is_none());
    }
}
