use sled::{TransactionError, Transactional};
use std::{
    io::{Read, Write},
    path::Path,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,
    #[error("database backend error")]
    BackendError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Block already present in DB")]
    BlockAlreadyPresent,
    #[error("the parent block is missing for the required write")]
    MissingParent,
}

#[derive(Clone)]
pub struct BlockStore {
    inner: sled::Db,
}

pub struct BlockInfo {
    pub id: Box<[u8]>,
    pub parent_id: Box<[u8]>,
    pub chain_length: u32,
}

mod tree {
    pub const BLOCKS: &str = "blocks";
    pub const INFO: &str = "info";
    pub const CHAIN_HEIGHT_INDEX: &str = "height_to_block_ids";
}

impl BlockInfo {
    fn serialize(&self) -> Vec<u8> {
        let mut w = Vec::new();

        let id_size = self.id.len() as u32;
        w.write_all(&id_size.to_le_bytes()).unwrap();

        w.write_all(&self.id).unwrap();

        let parent_id_size = self.id.len() as u32;
        w.write_all(&parent_id_size.to_le_bytes()).unwrap();

        w.write_all(&self.parent_id).unwrap();

        w.write_all(&self.chain_length.to_le_bytes()).unwrap();

        w
    }

    fn deserialize<R: Read>(mut r: R) -> Self {
        let mut id_size_bytes = [0u8; 4];
        r.read_exact(&mut id_size_bytes).unwrap();
        let id_size = u32::from_le_bytes(id_size_bytes);

        let mut id = vec![0u8; id_size as usize];
        r.read_exact(&mut id).unwrap();

        let mut parent_id_size_bytes = [0u8; 4];
        r.read_exact(&mut parent_id_size_bytes).unwrap();
        let parent_id_size = u32::from_le_bytes(parent_id_size_bytes);

        let mut parent_id = vec![0u8; parent_id_size as usize];
        r.read_exact(&mut parent_id).unwrap();

        let mut chain_length_bytes = [0u8; 4];
        r.read_exact(&mut chain_length_bytes).unwrap();
        let chain_length = u32::from_le_bytes(chain_length_bytes);

        BlockInfo {
            id: id.into_boxed_slice(),
            parent_id: parent_id.into_boxed_slice(),
            chain_length,
        }
    }
}

impl BlockStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = sled::open(path).map_err(|e| Error::BackendError(Box::new(e)))?;
        Ok(Self { inner })
    }

    /// Write a block to the store. The parent of the block must exist (unless
    /// it's the zero hash).
    pub fn put_block(&mut self, block: &[u8], block_info: BlockInfo) -> Result<(), Error> {
        let blocks = self
            .inner
            .open_tree(tree::BLOCKS)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let info = self
            .inner
            .open_tree(tree::INFO)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_to_block_ids = self
            .inner
            .open_tree(tree::CHAIN_HEIGHT_INDEX)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let result = (&blocks, &info, &height_to_block_ids).transaction(
            |(blocks, info, height_to_block_ids)| {
                if info.get(block_info.id.as_ref())?.is_some() {
                    return Ok(Err(Error::BlockAlreadyPresent));
                }

                // these if statementsa cannot be collapsed, this will result
                // into unnecessary DB reads if the optimization does not touch
                // it
                #[allow(clippy::collapsible_if)]
                {
                    if block_info.parent_id.as_ref()
                        != vec![0; block_info.parent_id.len()].as_slice()
                    {
                        if info.get(block_info.parent_id.as_ref())?.is_none() {
                            return Ok(Err(Error::MissingParent));
                        }
                    }
                }

                let mut height_index = block_info.chain_length.to_le_bytes().to_vec();
                height_index.extend_from_slice(block_info.id.as_ref());
                height_to_block_ids.insert(height_index, &[])?;

                blocks.insert(block_info.id.as_ref(), block)?;

                info.insert(block_info.id.as_ref(), block_info.serialize().as_slice())?;

                Ok(Ok(()))
            },
        );

        // Transactional impl for (&Tree, &Tree) implies the use of () as the
        // type for user-defined errors, so we have a workaround for that.
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(TransactionError::Storage(err)) => Err(Error::BackendError(Box::new(err))),
            Err(TransactionError::Abort(())) => unreachable!(),
        }
    }

    pub fn get_block(&mut self, block_hash: &[u8]) -> Result<Vec<u8>, Error> {
        let blocks = self
            .inner
            .open_tree(tree::BLOCKS)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        blocks
            .get(block_hash)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| {
                let mut v = Vec::new();
                v.extend_from_slice(&block_bin);
                v
            })
    }

    pub fn get_block_info(&mut self, block_hash: &[u8]) -> Result<BlockInfo, Error> {
        let info = self
            .inner
            .open_tree(tree::INFO)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        info.get(block_hash)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })
    }

    pub fn get_blocks_by_chain_length(&mut self, chain_length: u32) -> Result<Vec<Vec<u8>>, Error> {
        let blocks = self
            .inner
            .open_tree(tree::BLOCKS)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_to_block_ids = self
            .inner
            .open_tree(tree::CHAIN_HEIGHT_INDEX)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_index_prefix = chain_length.to_le_bytes();
        height_to_block_ids
            .scan_prefix(height_index_prefix)
            .map(|scan_result| {
                let block_hash = scan_result
                    .map(|(key, _)| Vec::from(&key[4..key.len()]))
                    .map_err(|err| Error::BackendError(Box::new(err)))?;

                blocks
                    .get(block_hash)
                    .map(|maybe_raw_block| {
                        let raw_block: &[u8] = &maybe_raw_block.unwrap();
                        Vec::from(raw_block)
                    })
                    .map_err(|err| Error::BackendError(Box::new(err)))
            })
            .collect()
    }

    pub fn put_tag(&mut self, tag_name: &str, block_hash: &[u8]) -> Result<(), Error> {
        let info = self
            .inner
            .open_tree(tree::INFO)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let tags = self
            .inner
            .open_tree("tags")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let result = (&info, &tags).transaction(move |(info, tags)| {
            if info.get(block_hash)?.is_none() {
                return Ok(Err(Error::BlockNotFound));
            }

            tags.insert(tag_name, block_hash)?;

            Ok(Ok(()))
        });

        // Transactional impl for (&Tree, &Tree) implies the use of () as the
        // type for user-defined errors, so we have a workaround for that.
        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(err)) => Err(err),
            Err(TransactionError::Storage(err)) => Err(Error::BackendError(Box::new(err))),
            Err(TransactionError::Abort(())) => unreachable!(),
        }
    }

    pub fn get_tag(&mut self, tag_name: &str) -> Result<Option<Vec<u8>>, Error> {
        let tags = self
            .inner
            .open_tree("tags")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tags.get(tag_name)
            .map(|maybe_id_bin| {
                maybe_id_bin.map(|id_bin| {
                    let mut v = Vec::new();
                    v.extend_from_slice(&id_bin);
                    v
                })
            })
            .map_err(|err| Error::BackendError(Box::new(err)))
    }

    pub fn block_exists(&mut self, block_hash: &[u8]) -> Result<bool, Error> {
        let info = self
            .inner
            .open_tree(tree::INFO)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        info.get(block_hash)
            .map(|maybe_block| maybe_block.is_some())
            .map_err(|err| Error::BackendError(Box::new(err)))
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
        if ancestor_id == descendent_id {
            return Ok(Some(0));
        }

        let info = self
            .inner
            .open_tree(tree::INFO)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let descendent: BlockInfo = info
            .get(descendent_id)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })?;

        if descendent_id == vec![0u8; descendent_id.len()].as_slice() {
            return Ok(Some(descendent.chain_length));
        }

        let mut ancestor_id = descendent.parent_id;

        let mut distance = 0;

        while let Some(ancestor) = info
            .get(ancestor_id.as_ref())
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })
        {
            distance += 1;
            if ancestor.id == ancestor_id {
                return Ok(Some(distance));
            }
            ancestor_id = ancestor.parent_id;
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
    let info = store
        .inner
        .open_tree(tree::INFO)
        .map_err(|err| Error::BackendError(Box::new(err)))?;

    let mut current = info
        .get(block_hash)
        .map_err(|err| Error::BackendError(Box::new(err)))
        .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
        .map(|block_info_bin| {
            let mut block_info_reader: &[u8] = &block_info_bin;
            BlockInfo::deserialize(&mut block_info_reader)
        })?;

    if distance >= current.chain_length {
        panic!(
            "distance {} > chain length {}",
            distance, current.chain_length
        );
    }

    let target = current.chain_length - distance;

    while target < current.chain_length {
        callback(&current);
        current = info
            .get(current.parent_id.as_ref())
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_info_bin| {
                let mut block_info_reader: &[u8] = &block_info_bin;
                BlockInfo::deserialize(&mut block_info_reader)
            })?;
    }

    Ok(current)
}

#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils {
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Copy)]
    pub struct BlockId(pub u64);

    static GLOBAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    impl BlockId {
        pub fn generate() -> Self {
            Self(GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
        }

        pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
            writer.write_all(&self.0.to_le_bytes())
        }

        pub fn serialize_as_vec(&self) -> Vec<u8> {
            let mut v = Vec::new();
            self.serialize(&mut v).unwrap();
            v
        }

        pub fn serialize_as_boxed_slice(&self) -> Box<[u8]> {
            self.serialize_as_vec().into_boxed_slice()
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Block {
        pub id: BlockId,
        pub parent: BlockId,
        pub chain_length: u32,
        pub data: Box<[u8]>,
    }

    impl Block {
        pub fn genesis(data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: BlockId(0),
                chain_length: 1,
                data: data.unwrap_or_default(),
            }
        }

        pub fn make_child(&self, data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: self.id,
                chain_length: self.chain_length,
                data: data.unwrap_or_default(),
            }
        }

        pub fn serialize<W: std::io::Write>(&self, mut writer: W) -> Result<(), std::io::Error> {
            writer.write_all(&self.id.0.to_le_bytes())?;
            writer.write_all(&self.parent.0.to_le_bytes())?;
            writer.write_all(&self.chain_length.to_le_bytes())?;
            writer.write_all(&(self.data.len() as u64).to_le_bytes())?;
            writer.write_all(&self.data)?;
            Ok(())
        }

        pub fn serialize_as_vec(&self) -> Vec<u8> {
            let mut v = Vec::new();
            self.serialize(&mut v).unwrap();
            v
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::test_utils::{Block, BlockId};
    use super::*;
    use rand_core::{OsRng, RngCore};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a [A]) -> &'a A {
        let s = rng.next_u32() as usize;
        // this doesn't need to be uniform
        &v[s % v.len()]
    }

    pub fn generate_chain<R: RngCore>(rng: &mut R, store: &mut BlockStore) -> Vec<Block> {
        let mut blocks = vec![];

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo {
            id: genesis_block.id.serialize_as_boxed_slice(),
            parent_id: genesis_block.parent.serialize_as_boxed_slice(),
            chain_length: genesis_block.chain_length,
        };
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        blocks.push(genesis_block);

        for _ in 0..10 {
            let mut parent_block = pick_from_vector(rng, &blocks).clone();
            let r = 1 + (rng.next_u32() % 9999);
            for _ in 0..r {
                let block = parent_block.make_child(None);
                let block_info = BlockInfo {
                    id: block.id.serialize_as_boxed_slice(),
                    parent_id: block.parent.serialize_as_boxed_slice(),
                    chain_length: block.chain_length,
                };
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
        let genesis_block_info = BlockInfo {
            id: genesis_block.id.serialize_as_boxed_slice(),
            parent_id: genesis_block.parent.serialize_as_boxed_slice(),
            chain_length: genesis_block.chain_length,
        };
        store
            .put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        let genesis_block_restored = store
            .get_block_info(&genesis_block.id.serialize_as_vec())
            .unwrap();

        assert_eq!(
            genesis_block.id.serialize_as_boxed_slice(),
            genesis_block_restored.id
        );
        assert_eq!(
            genesis_block.parent.serialize_as_boxed_slice(),
            genesis_block_restored.parent_id
        );
        assert_eq!(
            genesis_block.chain_length,
            genesis_block_restored.chain_length
        );

        store
            .put_tag("tip", &genesis_block.id.serialize_as_vec())
            .unwrap();
        assert_eq!(
            store.get_tag("tip").unwrap().unwrap(),
            genesis_block.id.serialize_as_vec()
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

            assert_eq!(ancestor_info.chain_length + distance, block.chain_length);
        }

        let blocks_per_test = blocks_fetched as f64 / nr_tests as f64;

        println!(
            "fetched {} intermediate blocks ({} per test), total distance {}",
            blocks_fetched, blocks_per_test, total_distance
        );

        assert!(blocks_per_test < 35.0);
    }

    #[test]
    fn simultaneous_read_write() {
        let mut rng = OsRng;
        let file = tempfile::TempDir::new().unwrap();
        let mut conn = BlockStore::new(file.path()).unwrap();

        let genesis_block = Block::genesis(None);
        let genesis_block_info = BlockInfo {
            id: genesis_block.id.serialize_as_boxed_slice(),
            parent_id: genesis_block.parent.serialize_as_boxed_slice(),
            chain_length: genesis_block.chain_length,
        };
        conn.put_block(&genesis_block.serialize_as_vec(), genesis_block_info)
            .unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            blocks.push(block.clone());
            let block_info = BlockInfo {
                id: block.id.serialize_as_boxed_slice(),
                parent_id: block.parent.serialize_as_boxed_slice(),
                chain_length: block.chain_length,
            };
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
                let block_info = BlockInfo {
                    id: block.id.serialize_as_boxed_slice(),
                    parent_id: block.parent.serialize_as_boxed_slice(),
                    chain_length: block.chain_length,
                };
                conn.put_block(&block.serialize_as_vec(), block_info)
                    .unwrap()
            }
        });

        thread_1.join().unwrap();
        thread_2.join().unwrap();
    }
}
