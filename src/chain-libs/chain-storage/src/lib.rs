use chain_core::property::{BlockId, Deserialize, Serialize};
use sled::{TransactionError, Transactional};
use std::{io::Write, marker::PhantomData, path::Path};
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

pub trait Block: Serialize + Deserialize {
    type Id: BlockId;

    fn id(&self) -> Self::Id;
    fn parent_id(&self) -> Self::Id;
    fn chain_length(&self) -> u32;
}

#[derive(Clone)]
pub struct BlockStore<B> {
    inner: sled::Db,
    dummy: PhantomData<B>,
}

impl<B> BlockStore<B>
where
    B: Block,
{
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let inner = sled::open(path).map_err(|e| Error::BackendError(Box::new(e)))?;
        Ok(Self {
            inner,
            dummy: PhantomData,
        })
    }

    /// Write a block to the store. The parent of the block must exist (unless
    /// it's the zero hash).
    pub fn put_block(&mut self, block: &B) -> Result<(), Error> {
        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_to_block_ids = self
            .inner
            .open_tree("height_to_block_ids")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let result =
            (&blocks, &height_to_block_ids).transaction(|(blocks, height_to_block_ids)| {
                let block_hash = block.id().serialize_as_vec().unwrap();

                if blocks.get(block_hash.clone())?.is_some() {
                    return Ok(Err(Error::BlockAlreadyPresent));
                }

                if block.parent_id() != B::Id::zero() {
                    let parent_id = block.parent_id().serialize_as_vec().unwrap();
                    if blocks.get(parent_id)?.is_none() {
                        return Ok(Err(Error::MissingParent));
                    }
                }

                let height_index = block.chain_length().to_le_bytes();
                let mut ids_new = vec![];
                let ids_old = height_to_block_ids
                    .get(height_index)?
                    .unwrap_or(sled::IVec::default());
                ids_new.write_all(&ids_old).unwrap();
                ids_new.write_all(&block_hash).unwrap();
                height_to_block_ids.insert(&height_index, ids_new)?;

                blocks.insert(block_hash, block.serialize_as_vec().unwrap())?;

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

    pub fn get_block(&mut self, block_hash: &B::Id) -> Result<B, Error> {
        let block_hash = block_hash.serialize_as_vec().unwrap();

        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        blocks
            .get(block_hash)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())
    }

    pub fn get_blocks_by_chain_length(&mut self, chain_length: u32) -> Result<Vec<B>, Error> {
        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_to_block_ids = self
            .inner
            .open_tree("height_to_block_ids")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let height_index = chain_length.to_le_bytes();
        let ids_raw = height_to_block_ids
            .get(height_index)
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .unwrap_or(sled::IVec::default());
        let mut ids_raw_reader: &[u8] = &ids_raw;

        let mut ids = vec![];

        while ids_raw_reader.len() != 0 {
            ids.push(B::Id::deserialize(&mut ids_raw_reader).unwrap());
        }

        ids.iter()
            .map(|id| {
                blocks
                    .get(id.serialize_as_vec().unwrap())
                    .map(|maybe_raw_block| B::deserialize(&maybe_raw_block.unwrap()[..]).unwrap())
                    .map_err(|err| Error::BackendError(Box::new(err)))
            })
            .collect()
    }

    pub fn put_tag(&mut self, tag_name: &str, block_hash: &B::Id) -> Result<(), Error> {
        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let tags = self
            .inner
            .open_tree("tags")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let result = (&blocks, &tags).transaction(move |(blocks, tags)| {
            let block_hash = block_hash.serialize_as_vec().unwrap();

            if blocks.get(block_hash.clone())?.is_none() {
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

    pub fn get_tag(&mut self, tag_name: &str) -> Result<Option<B::Id>, Error> {
        let tags = self
            .inner
            .open_tree("tags")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tags.get(tag_name)
            .map(|maybe_id_bin| maybe_id_bin.map(|id_bin| B::Id::deserialize(&id_bin[..]).unwrap()))
            .map_err(|err| Error::BackendError(Box::new(err)))
    }

    pub fn block_exists(&mut self, block_hash: &B::Id) -> Result<bool, Error> {
        let block_hash = block_hash.serialize_as_vec().unwrap();

        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        blocks
            .get(block_hash)
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
        ancestor_id: &B::Id,
        descendent_id: &B::Id,
    ) -> Result<Option<u32>, Error> {
        if ancestor_id == descendent_id {
            return Ok(Some(0));
        }

        let descendent_id_bin = descendent_id.serialize_as_vec().unwrap();

        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let descendent = blocks
            .get(descendent_id_bin)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())?;

        if descendent_id == &B::Id::zero() {
            return Ok(Some(descendent.chain_length()));
        }

        let mut ancestor_id_bin = descendent.parent_id().serialize_as_vec().unwrap();

        let mut distance = 0;

        while let Some(ancestor) = blocks
            .get(ancestor_id_bin)
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())
        {
            distance += 1;
            if &ancestor.id() == ancestor_id {
                return Ok(Some(distance));
            }
            ancestor_id_bin = ancestor.parent_id().serialize_as_vec().unwrap();
        }

        Ok(None)
    }

    pub fn get_nth_ancestor(&mut self, block_hash: &B::Id, distance: u32) -> Result<B, Error> {
        let block_hash = block_hash.serialize_as_vec().unwrap();

        let blocks = self
            .inner
            .open_tree("blocks")
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let descendent = blocks
            .get(block_hash)
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())?;

        if distance == 0 {
            return Ok(descendent);
        }

        if distance >= descendent.chain_length() {
            return Err(Error::BlockNotFound);
        }

        let mut ancestor_id_bin = descendent.parent_id().serialize_as_vec().unwrap();

        let mut actual_distance = 0;

        while let Some(ancestor) = blocks
            .get(ancestor_id_bin)
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())
        {
            actual_distance += 1;
            if actual_distance == distance {
                return Ok(ancestor);
            }
            ancestor_id_bin = ancestor.parent_id().serialize_as_vec().unwrap();
        }

        Err(Error::BlockNotFound)
    }
}

/// Like `BlockStore::get_nth_ancestor`, but calls the closure 'callback' with
/// each intermediate block encountered while travelling from
/// 'block_hash' to its n'th ancestor.
///
/// The travelling algorithm uses back links to skip over parts of the chain,
/// so the callback will not be invoked for all blocks in the linear sequence.
pub fn for_path_to_nth_ancestor<B, F>(
    store: &mut BlockStore<B>,
    block_hash: &B::Id,
    distance: u32,
    mut callback: F,
) -> Result<B, Error>
where
    B: Block,
    F: FnMut(&B),
{
    let block_hash = block_hash.serialize_as_vec().unwrap();

    let blocks = store
        .inner
        .open_tree("blocks")
        .map_err(|err| Error::BackendError(Box::new(err)))?;

    let mut current = blocks
        .get(block_hash)
        .map_err(|err| Error::BackendError(Box::new(err)))
        .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
        .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())?;

    if distance >= current.chain_length() {
        return Err(Error::BlockNotFound);
    }

    let target = current.chain_length() - distance;

    while target < current.chain_length() {
        callback(&current);
        current = blocks
            .get(current.parent_id().serialize_as_vec().unwrap())
            .map_err(|err| Error::BackendError(Box::new(err)))
            .and_then(|maybe_block| maybe_block.ok_or(Error::BlockNotFound))
            .map(|block_bin| B::deserialize(&block_bin[..]).unwrap())?;
    }

    Ok(current)
}

#[cfg(any(test, feature = "with-bench"))]
pub mod test_utils {
    use chain_core::packer::*;
    use chain_core::property::{BlockDate as _, BlockId as _};
    use std::sync::atomic::{AtomicU64, Ordering};

    #[derive(Debug, Clone, Hash, Ord, PartialOrd, Eq, PartialEq, Copy)]
    pub struct BlockId(pub u64);

    static GLOBAL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

    impl BlockId {
        pub fn generate() -> Self {
            Self(GLOBAL_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
        }
    }

    impl chain_core::property::BlockId for BlockId {
        fn zero() -> Self {
            Self(0)
        }
    }

    impl chain_core::property::Serialize for BlockId {
        type Error = std::io::Error;

        fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
            let mut codec = Codec::new(writer);
            codec.put_u64(self.0)
        }
    }

    impl chain_core::property::Deserialize for BlockId {
        type Error = std::io::Error;

        fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
            let mut codec = Codec::new(reader);
            Ok(Self(codec.get_u64()?))
        }
    }

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Copy)]
    pub struct BlockDate(u32, u32);

    impl chain_core::property::BlockDate for BlockDate {
        fn from_epoch_slot_id(epoch: u32, slot_id: u32) -> Self {
            Self(epoch, slot_id)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub struct Block {
        id: BlockId,
        parent: BlockId,
        date: BlockDate,
        chain_length: u32,
        data: Box<[u8]>,
    }

    impl Block {
        pub fn genesis(data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: BlockId::zero(),
                date: BlockDate::from_epoch_slot_id(0, 0),
                chain_length: 1,
                data: data.unwrap_or_default(),
            }
        }

        pub fn make_child(&self, data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: self.id,
                date: BlockDate::from_epoch_slot_id(self.date.0, self.date.1 + 1),
                chain_length: self.chain_length,
                data: data.unwrap_or_default(),
            }
        }
    }

    impl super::Block for Block {
        type Id = BlockId;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn parent_id(&self) -> Self::Id {
            self.parent
        }

        fn chain_length(&self) -> u32 {
            self.chain_length
        }
    }

    impl chain_core::property::Serialize for Block {
        type Error = std::io::Error;

        fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
            let mut codec = Codec::new(writer);
            codec.put_u64(self.id.0)?;
            codec.put_u64(self.parent.0)?;
            codec.put_u32(self.date.0)?;
            codec.put_u32(self.date.1)?;
            codec.put_u32(self.chain_length)?;
            codec.put_u64(self.data.len() as u64)?;
            codec.put_bytes(&self.data)?;
            Ok(())
        }
    }

    impl chain_core::property::Deserialize for Block {
        type Error = std::io::Error;

        fn deserialize<R: std::io::BufRead>(reader: R) -> Result<Self, Self::Error> {
            let mut codec = Codec::new(reader);
            Ok(Self {
                id: BlockId(codec.get_u64()?),
                parent: BlockId(codec.get_u64()?),
                date: BlockDate(codec.get_u32()?, codec.get_u32()?),
                chain_length: codec.get_u32()?,
                data: {
                    let length = codec.get_u64()?;
                    codec.get_bytes(length as usize)?.into_boxed_slice()
                },
            })
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::test_utils::{Block, BlockId};
    use super::Block as _;
    use super::*;
    use chain_core::property::BlockId as _;
    use rand_core::{OsRng, RngCore};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a [A]) -> &'a A {
        let s = rng.next_u32() as usize;
        // this doesn't need to be uniform
        &v[s % v.len()]
    }

    pub fn generate_chain<R: RngCore>(rng: &mut R, store: &mut BlockStore<Block>) -> Vec<Block> {
        let mut blocks = vec![];

        let genesis_block = Block::genesis(None);
        store.put_block(&genesis_block).unwrap();
        blocks.push(genesis_block);

        for _ in 0..10 {
            let mut parent_block = pick_from_vector(rng, &blocks).clone();
            let r = 1 + (rng.next_u32() % 9999);
            for _ in 0..r {
                let block = parent_block.make_child(None);
                store.put_block(&block).unwrap();
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

        match store.put_tag("tip", &BlockId::zero()) {
            Err(Error::BlockNotFound) => {}
            err => panic!(err),
        }

        let genesis_block = Block::genesis(None);
        store.put_block(&genesis_block).unwrap();
        let genesis_block_restored = store.get_block(&genesis_block.id()).unwrap();
        assert_eq!(genesis_block, genesis_block_restored);

        store.put_tag("tip", &genesis_block.id()).unwrap();
        assert_eq!(store.get_tag("tip").unwrap().unwrap(), genesis_block.id());
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
            assert_eq!(&store.get_block(&block.id()).unwrap(), block);

            let distance = rng.next_u32() % block.chain_length();
            total_distance += distance;

            let ancestor_info = for_path_to_nth_ancestor(&mut store, &block.id(), distance, |_| {
                blocks_fetched += 1;
            })
            .unwrap();

            assert_eq!(
                ancestor_info.chain_length() + distance,
                block.chain_length()
            );

            let ancestor = store.get_block(&ancestor_info.id()).unwrap();

            assert_eq!(ancestor.chain_length() + distance, block.chain_length());
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
        conn.put_block(&genesis_block).unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            blocks.push(block.clone());
            conn.put_block(&block).unwrap()
        }

        let mut conn_1 = conn.clone();
        let blocks_1 = blocks.clone();

        let thread_1 = std::thread::spawn(move || {
            for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
                let block_id = blocks_1
                    .get(rng.next_u32() as usize % blocks_1.len())
                    .unwrap()
                    .id();
                conn_1.get_block(&block_id).unwrap();
            }
        });

        let thread_2 = std::thread::spawn(move || {
            for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
                let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
                let block = last_block.make_child(None);
                conn.put_block(&block).unwrap();
            }
        });

        thread_1.join().unwrap();
        thread_2.join().unwrap();
    }
}
