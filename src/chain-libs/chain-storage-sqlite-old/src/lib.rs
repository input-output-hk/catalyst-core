use chain_core::property::{Block, BlockId, Serialize};
use rusqlite::{types::Value, Connection};
use std::{error, fmt, path::{Path, PathBuf}};

#[derive(Debug)]
pub enum Error {
    BlockNotFound, // FIXME: add BlockId
    CannotIterate,
    BackendError(Box<dyn std::error::Error + Send + Sync>),
    Block0InFuture,
    BlockAlreadyPresent,
    MissingParent,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BlockNotFound => write!(f, "block not found"),
            Error::CannotIterate => write!(f, "cannot iterate between the 2 given blocks"),
            Error::BackendError(err) => write!(f, "{}", err),
            Error::Block0InFuture => write!(f, "block0 is in the future"),
            Error::BlockAlreadyPresent => write!(f, "Block already present in DB"),
            Error::MissingParent => write!(f, "the parent block is missing for the required write"),
        }
    }
}

impl error::Error for Error {}

#[derive(Clone, Debug)]
pub struct BackLink<Id: BlockId> {
    /// The distance to this ancestor.
    pub distance: u64,
    /// The hash of the ancestor.
    pub block_hash: Id,
}

#[derive(Clone, Debug)]
pub struct BlockInfo<Id: BlockId> {
    pub block_hash: Id,

    /// Length of the chain. I.e. a block whose parent is the zero
    /// hash has depth 1, its children have depth 2, and so on. Note
    /// that there is no block with depth 0 because there is no block
    /// with the zero hash.
    // FIXME: rename to chain_length
    pub depth: u64,

    /// One or more ancestors of this block. Must include at least the
    /// parent, but may include other ancestors to enable efficient
    /// random access in get_nth_ancestor().
    pub back_links: Vec<BackLink<Id>>,
}

impl<Id: BlockId> BlockInfo<Id> {
    pub fn parent_id(&self) -> Id {
        self.back_links
            .iter()
            .find(|x| x.distance == 1)
            .unwrap()
            .block_hash
            .clone()
    }
}

enum StoreType {
    Memory,
    File(PathBuf),
}

pub struct SQLiteBlockStore {
    store_type: StoreType,
    // An optional connection to be always held open. This is a workaround to
    // prevent an in-memory storage from resetting, because it is getting reset
    // once the last open connection was removed.
    persistent_connection: Option<Connection>,
}

// persistent_connection does not implement Sync but is never actually used
// which makes it safe to be shared
unsafe impl Sync for SQLiteBlockStore {}

pub struct SQLiteBlockStoreConnection<B>
where
    B: Block,
{
    inner: Connection,
    dummy: std::marker::PhantomData<B>,
}

impl SQLiteBlockStore {
    pub fn memory() -> Self {
        Self::init(StoreType::Memory)
    }

    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        Self::init(StoreType::File(path.as_ref().to_path_buf()))
    }

    fn init(store_type: StoreType) -> Self {
        let mut store = Self {
            store_type,
            persistent_connection: None,
        };

        let connection = store.connect_internal().unwrap();

        connection
            .execute_batch(
                r#"
                  begin;

                  create table if not exists BlockInfo (
                    hash blob primary key,
                    depth integer not null,
                    parent blob not null,
                    fast_distance blob,
                    fast_hash blob,
                    foreign key(hash) references Blocks(hash)
                  );

                  create table if not exists Blocks (
                    hash blob primary key,
                    block blob not null
                  );

                  create table if not exists Tags (
                    name text primary key,
                    hash blob not null,
                    foreign key(hash) references BlockInfo(hash)
                  );

                  commit;
                "#,
            )
            .unwrap();

        connection
            .execute_batch("pragma journal_mode = WAL")
            .unwrap();

        store.persistent_connection = Some(connection);

        store
    }

    fn connect_internal(&self) -> Result<Connection, Error> {
        // Shared cache should be always enabled for in-memory databases so that
        // all connections in a pool access the same database. Otherwise each
        // connection has its own database which leads to bugs, because only one
        // of those databases will have a schema set.
        match &self.store_type {
            StoreType::Memory => Connection::open("file::memory:?cache=shared"),
            StoreType::File(path) => Connection::open(path),
        }
        .map_err(|err| Error::BackendError(Box::new(err)))
    }

    pub fn connect<B>(&self) -> Result<SQLiteBlockStoreConnection<B>, Error>
    where
        B: Block,
    {
        Ok(SQLiteBlockStoreConnection {
            inner: self.connect_internal()?,
            dummy: std::marker::PhantomData,
        })
    }
}

impl<B> SQLiteBlockStoreConnection<B>
where
    B: Block,
{
    pub fn ping(&self) -> Result<(), Error> {
        self.inner
            .execute_batch("")
            .map_err(|e| Error::BackendError(Box::new(e)))
    }

    /// Write a block to the store. The parent of the block must exist
    /// (unless it's the zero hash).
    ///
    /// The default implementation computes a BlockInfo structure with
    /// back_links set to ensure O(lg n) seek time in
    /// get_nth_ancestor(), and calls put_block_internal() to do the
    /// actual write.
    pub fn put_block(&mut self, block: &B) -> Result<(), Error> {
        let block_hash = block.id();

        if self.block_exists(&block_hash)? {
            return Ok(());
        }

        let parent_hash = block.parent_id();

        // Always include a link to the parent.
        let mut back_links = vec![BackLink {
            distance: 1,
            block_hash: parent_hash.clone(),
        }];

        let depth = if parent_hash == B::Id::zero() {
            1
        } else {
            let parent_info = self.get_block_info(&parent_hash).map_err(|e| match e {
                Error::BlockNotFound => Error::MissingParent,
                e => e,
            })?;
            assert!(parent_info.depth > 0);
            let depth = 1 + parent_info.depth;
            let fast_link = compute_fast_link(depth);
            //println!("from {} -> {}", depth, fast_link);
            let distance = depth - fast_link;
            if distance != 1 && fast_link > 0 {
                let far_block_info = self.get_nth_ancestor(&parent_hash, depth - 1 - fast_link)?;
                back_links.push(BackLink {
                    distance,
                    block_hash: far_block_info.block_hash,
                })
            }

            depth
        };

        let block_info = BlockInfo {
            block_hash: block_hash.clone(),
            depth,
            back_links,
        };

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let worked = tx
            .prepare_cached("insert into Blocks (hash, block) values(?, ?)")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .execute(&[
                &block_info.block_hash.serialize_as_vec().unwrap()[..],
                &block.serialize_as_vec().unwrap()[..],
            ])
            .map(|_| true)
            .or_else(|err| match err {
                rusqlite::Error::SqliteFailure(error, _) => {
                    if error.code == rusqlite::ErrorCode::ConstraintViolation {
                        Ok(false)
                    } else {
                        Err(err)
                    }
                }
                _ => Err(err),
            })
            .map_err(|err| Error::BackendError(Box::new(err)))?;
        if !worked {
            return Err(Error::BlockAlreadyPresent);
        }

        let parent = block_info
            .back_links
            .iter()
            .find(|x| x.distance == 1)
            .unwrap();

        let (fast_distance, fast_hash) =
            match block_info.back_links.iter().find(|x| x.distance != 1) {
                Some(fast_link) => (
                    Value::Integer(fast_link.distance as i64),
                    Value::Blob(fast_link.block_hash.serialize_as_vec().unwrap()),
                ),
                None => (Value::Null, Value::Null),
            };

        tx
            .prepare_cached("insert into BlockInfo (hash, depth, parent, fast_distance, fast_hash) values(?, ?, ?, ?, ?)")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .execute(&[
                Value::Blob(block_info.block_hash.serialize_as_vec().unwrap()),
                Value::Integer(block_info.depth as i64),
                Value::Blob(parent.block_hash.serialize_as_vec().unwrap()),
                fast_distance,
                fast_hash,
            ])
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tx.commit()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Ok(())
    }

    pub fn get_block(&self, block_hash: &B::Id) -> Result<(B, BlockInfo<B::Id>), Error> {
        let blk = self
            .inner
            .prepare_cached("select block from Blocks where hash = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[&block_hash.serialize_as_vec().unwrap()[..]], |row| {
                let x: Vec<u8> = row.get(0);
                B::deserialize(&x[..]).unwrap()
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?;

        let info = self.get_block_info(block_hash)?;

        Ok((blk, info))
    }

    pub fn get_block_info(&self, block_hash: &B::Id) -> Result<BlockInfo<B::Id>, Error> {
        self
            .inner
            .prepare_cached(
                "select depth, parent, fast_distance, fast_hash from BlockInfo where hash = ?",
            )
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[&block_hash.serialize_as_vec().unwrap()[..]], |row| {
                let mut back_links = vec![BackLink {
                    distance: 1,
                    block_hash: blob_to_hash(row.get(1)),
                }];

                let fast_distance: Option<i64> = row.get(2);
                if let Some(fast_distance) = fast_distance {
                    back_links.push(BackLink {
                        distance: fast_distance as u64,
                        block_hash: blob_to_hash(row.get(3)),
                    });
                }

                let depth: i64 = row.get(0);

                BlockInfo {
                    block_hash: block_hash.clone(),
                    depth: depth as u64,
                    back_links,
                }
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })
    }

    pub fn put_tag(&mut self, tag_name: &str, block_hash: &B::Id) -> Result<(), Error> {
        match self
            .inner
            .prepare_cached("insert or replace into Tags (name, hash) values(?, ?)")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .execute(&[
                Value::Text(tag_name.to_string()),
                Value::Blob(block_hash.serialize_as_vec().unwrap()),
            ]) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _))
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(Error::BlockNotFound)
            }
            Err(err) => Err(Error::BackendError(Box::new(err))),
        }
    }

    pub fn get_tag(&self, tag_name: &str) -> Result<Option<B::Id>, Error> {
        match self
            .inner
            .prepare_cached("select hash from Tags where name = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[&tag_name], |row| blob_to_hash(row.get(0)))
        {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(Error::BackendError(Box::new(err))),
        }
    }

    /// Check whether a block exists.
    pub fn block_exists(&self, block_hash: &B::Id) -> Result<bool, Error> {
        match self.get_block_info(block_hash) {
            Ok(_) => Ok(true),
            Err(Error::BlockNotFound) => Ok(false),
            Err(err) => Err(err),
        }
    }

    // Determine whether block 'ancestor' is an ancestor of block 'descendent'
    ///
    /// Returned values:
    /// - `Ok(Some(dist))` - `ancestor` is ancestor of `descendent`
    ///     and there are `dist` blocks between them
    /// - `Ok(None)` - `ancestor` is not ancestor of `descendent`
    /// - `Err(error)` - `ancestor` or `descendent` was not found
    pub fn is_ancestor(&self, ancestor: &B::Id, descendent: &B::Id) -> Result<Option<u64>, Error> {
        // Optimization.
        if ancestor == descendent {
            return Ok(Some(0));
        }

        let descendent = self.get_block_info(&descendent)?;

        if ancestor == &B::Id::zero() {
            return Ok(Some(descendent.depth));
        }

        let ancestor = self.get_block_info(&ancestor)?;

        // Bail out right away if the "descendent" does not have a
        // higher depth.
        if descendent.depth <= ancestor.depth {
            return Ok(None);
        }

        // Seek back from the descendent to check whether it has the
        // ancestor at the expected place.
        let info =
            self.get_nth_ancestor(&descendent.block_hash, descendent.depth - ancestor.depth)?;

        if info.block_hash == ancestor.block_hash {
            Ok(Some(descendent.depth - ancestor.depth))
        } else {
            Ok(None)
        }
    }

    pub fn get_nth_ancestor(
        &self,
        block_hash: &B::Id,
        distance: u64,
    ) -> Result<BlockInfo<B::Id>, Error> {
        for_path_to_nth_ancestor(self, block_hash, distance, |_| {})
    }
}

fn blob_to_hash<Id: BlockId>(blob: Vec<u8>) -> Id {
    Id::deserialize(&blob[..]).unwrap()
}

/// Like `BlockStore::get_nth_ancestor`, but calls the closure 'callback' with
/// each intermediate block encountered while travelling from
/// 'block_hash' to its n'th ancestor.
///
/// The travelling algorithm uses back links to skip over parts of the chain,
/// so the callback will not be invoked for all blocks in the linear sequence.
pub fn for_path_to_nth_ancestor<B, F>(
    store: &SQLiteBlockStoreConnection<B>,
    block_hash: &B::Id,
    distance: u64,
    mut callback: F,
) -> Result<BlockInfo<B::Id>, Error>
where
    B: Block,
    F: FnMut(&BlockInfo<B::Id>),
{
    let mut cur_block_info = store.get_block_info(block_hash)?;

    if distance >= cur_block_info.depth {
        // FIXME: return error
        panic!(
            "distance {} > chain length {}",
            distance, cur_block_info.depth
        );
    }

    let target = cur_block_info.depth - distance;

    // Travel back through the chain using the back links until we
    // reach the desired block.
    while target < cur_block_info.depth {
        // We're not there yet. Use the back link that takes us
        // furthest back in the chain, without going beyond the
        // block we're looking for.
        let best_link = cur_block_info
            .back_links
            .iter()
            .filter(|x| cur_block_info.depth - target >= x.distance)
            .max_by_key(|x| x.distance)
            .unwrap()
            .clone();
        callback(&cur_block_info);
        cur_block_info = store.get_block_info(&best_link.block_hash)?;
    }

    assert_eq!(target, cur_block_info.depth);

    Ok(cur_block_info)
}

/// Compute the fast link for a block with a given depth. Successive
/// blocks make a depth jump equal to differents powers of two, minus
/// 1, e.g. 1, 3, 7, 15, 31, ...
fn compute_fast_link(depth: u64) -> u64 {
    let order = depth % 32;
    let distance = if order == 0 { 1 } else { (1 << order) - 1 };
    if distance < depth {
        depth - distance
    } else {
        0
    }
}

/// Return an iterator that yields block info for the blocks of `store` in
/// the half-open range `(from, to]`. `from` must be an ancestor
/// of `to` and may be the zero hash.
pub fn iterate_range<'store, B>(
    store: &'store SQLiteBlockStoreConnection<B>,
    from: &B::Id,
    to: &B::Id,
) -> Result<BlockIterator<'store, B>, Error>
where
    B: Block,
{
    // FIXME: put blocks loaded by is_ancestor into pending_infos.
    match store.is_ancestor(from, to)? {
        None => Err(Error::CannotIterate),
        Some(distance) => {
            let to_info = store.get_block_info(&to)?;
            Ok(BlockIterator {
                store: store,
                to_depth: to_info.depth,
                cur_depth: to_info.depth - distance,
                pending_infos: vec![to_info],
            })
        }
    }
}

pub struct BlockIterator<'store, B>
where
    B: Block,
{
    store: &'store SQLiteBlockStoreConnection<B>,
    to_depth: u64,
    cur_depth: u64,
    pending_infos: Vec<BlockInfo<B::Id>>,
}

impl<'store, B> Iterator for BlockIterator<'store, B>
where
    B: Block,
{
    type Item = Result<BlockInfo<B::Id>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_depth >= self.to_depth {
            None
        } else {
            self.cur_depth += 1;

            let block_info = self.pending_infos.pop().unwrap();

            if block_info.depth == self.cur_depth {
                // We've seen this block on a previous ancestor traversal.
                Some(Ok(block_info))
            } else {
                // We don't have this block yet, so search back from
                // the furthest block that we do have.
                assert!(self.cur_depth < block_info.depth);
                let depth = block_info.depth;
                let parent = block_info.parent_id();
                self.pending_infos.push(block_info);
                Some(for_path_to_nth_ancestor(
                    self.store,
                    &parent,
                    depth - self.cur_depth - 1,
                    |new_info| {
                        self.pending_infos.push(new_info.clone());
                    },
                ))
            }
        }
    }
}

#[cfg(any(test, feature = "with-bench"))]
pub mod tests {
    use super::*;
    use chain_core::packer::*;
    use chain_core::property::{Block as _, BlockDate as _, BlockId as _};
    use rand_core::{OsRng, RngCore};
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU64, Ordering};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;

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
        chain_length: ChainLength,
    }

    impl Block {
        pub fn genesis() -> Self {
            Self {
                id: BlockId::generate(),
                parent: BlockId::zero(),
                date: BlockDate::from_epoch_slot_id(0, 0),
                chain_length: ChainLength(1),
            }
        }

        pub fn make_child(&self) -> Self {
            Self {
                id: BlockId::generate(),
                parent: self.id,
                date: BlockDate::from_epoch_slot_id(self.date.0, self.date.1 + 1),
                chain_length: ChainLength(self.chain_length.0 + 1),
            }
        }
    }

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Copy)]
    pub struct ChainLength(pub u64);

    impl chain_core::property::ChainLength for ChainLength {
        fn next(&self) -> Self {
            Self(self.0 + 1)
        }
    }

    impl chain_core::property::Block for Block {
        type Id = BlockId;
        type Date = BlockDate;
        type ChainLength = ChainLength;
        type Version = u8;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn parent_id(&self) -> Self::Id {
            self.parent
        }

        fn date(&self) -> Self::Date {
            self.date
        }

        fn version(&self) -> Self::Version {
            0
        }

        fn chain_length(&self) -> Self::ChainLength {
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
            codec.put_u64(self.chain_length.0)?;
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
                chain_length: ChainLength(codec.get_u64()?),
            })
        }
    }

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a Vec<A>) -> &'a A {
        let s = rng.next_u32() as usize;
        // this doesn't need to be uniform
        &v[s % v.len()]
    }

    pub fn generate_chain<R: RngCore>(
        rng: &mut R,
        store: &mut SQLiteBlockStoreConnection<Block>,
    ) -> Vec<Block> {
        let mut blocks = vec![];

        let genesis_block = Block::genesis();
        store.put_block(&genesis_block).unwrap();
        blocks.push(genesis_block);

        for _ in 0..10 {
            let mut parent_block = pick_from_vector(rng, &blocks).clone();
            let r = 1 + (rng.next_u32() % 9999);
            for _ in 0..r {
                let block = parent_block.make_child();
                store.put_block(&block).unwrap();
                parent_block = block.clone();
                blocks.push(block);
            }
        }

        blocks
    }

    #[test]
    pub fn test_put_get() {
        let mut store =
            SQLiteBlockStore::file("file:test_put_get?mode=memory&cache=shared")
                .connect()
                .unwrap();
        assert!(store.get_tag("tip").unwrap().is_none());

        match store.put_tag("tip", &BlockId::zero()) {
            Err(Error::BlockNotFound) => {}
            err => panic!(err),
        }

        let genesis_block = Block::genesis();
        store.put_block(&genesis_block).unwrap();
        let (genesis_block_restored, block_info) = store.get_block(&genesis_block.id()).unwrap();
        assert_eq!(genesis_block, genesis_block_restored);
        assert_eq!(block_info.block_hash, genesis_block.id());
        assert_eq!(block_info.depth, genesis_block.chain_length().0);
        assert_eq!(block_info.back_links.len(), 1);
        assert_eq!(block_info.parent_id(), BlockId::zero());

        store.put_tag("tip", &genesis_block.id()).unwrap();
        assert_eq!(store.get_tag("tip").unwrap().unwrap(), genesis_block.id());
    }

    #[test]
    pub fn test_nth_ancestor() {
        let mut rng = OsRng;
        let mut store =
            SQLiteBlockStore::file("file:test_nth_ancestor?mode=memory&cache=shared")
                .connect()
                .unwrap();
        let blocks = generate_chain(&mut rng, &mut store);

        let mut blocks_fetched = 0;
        let mut total_distance = 0;
        let nr_tests = 1000;

        for _ in 0..nr_tests {
            let block = pick_from_vector(&mut rng, &blocks);
            assert_eq!(&store.get_block(&block.id()).unwrap().0, block);

            let distance = rng.next_u64() % block.chain_length().0;
            total_distance += distance;

            let ancestor_info = for_path_to_nth_ancestor(&store, &block.id(), distance, |_| {
                blocks_fetched += 1;
            })
            .unwrap();

            assert_eq!(ancestor_info.depth + distance, block.chain_length().0);

            let ancestor = store.get_block(&ancestor_info.block_hash).unwrap().0;

            assert_eq!(ancestor.chain_length().0 + distance, block.chain_length().0);
        }

        let blocks_per_test = blocks_fetched as f64 / nr_tests as f64;

        println!(
            "fetched {} intermediate blocks ({} per test), total distance {}",
            blocks_fetched, blocks_per_test, total_distance
        );

        assert!(blocks_per_test < 35.0);
    }

    #[test]
    pub fn test_iterate_range() {
        let mut rng = OsRng;
        let mut store =
            SQLiteBlockStore::file("file:test_iterate_range?mode=memory&cache=shared")
                .connect()
                .unwrap();
        let blocks = generate_chain(&mut rng, &mut store);

        let blocks_by_id: HashMap<BlockId, &Block> = blocks.iter().map(|b| (b.id(), b)).collect();

        for _ in 0..1000 {
            let from = pick_from_vector(&mut rng, &blocks);
            let to = pick_from_vector(&mut rng, &blocks);

            match iterate_range(&store, &from.id(), &to.id()) {
                Ok(iter) => {
                    let mut prev = from.id();
                    for block_info in iter {
                        let block_info = block_info.unwrap();
                        assert_eq!(block_info.parent_id(), prev);
                        prev = block_info.block_hash;
                    }
                    assert_eq!(prev, to.id());
                }
                Err(Error::CannotIterate) => {
                    // Check that 'from' really isn't an ancestor of 'to'.
                    let mut cur = to.id();
                    while cur != BlockId::zero() {
                        assert_ne!(cur, from.id());
                        cur = blocks_by_id[&cur].parent_id();
                    }
                }
                Err(_) => panic!(),
            }
        }
    }

    #[test]
    fn simultaneous_read_write() {
        let mut rng = OsRng;
        let store =
            SQLiteBlockStore::file("file:test_simultaneous_read_write?mode=memory&cache=shared");

        let mut conn = store.connect::<Block>().unwrap();

        let genesis_block = Block::genesis();
        conn.put_block(&genesis_block).unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child();
            blocks.push(block.clone());
            conn.put_block(&block).unwrap()
        }

        let conn_1 = store.connect::<Block>().unwrap();
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
                let block = last_block.make_child();
                conn.put_block(&block).unwrap();
            }
        });

        thread_1.join().unwrap();
        thread_2.join().unwrap();
    }
}
