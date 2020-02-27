use chain_core::property::{Block, BlockId, Serialize};
use rusqlite::{types::Value, Connection, TransactionBehavior};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound, // FIXME: add BlockId
    #[error("cannot iterate between the 2 given blocks")]
    CannotIterate,
    #[error("database backend error")]
    BackendError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("block0 is in the future")]
    Block0InFuture,
    #[error("Block already present in DB")]
    BlockAlreadyPresent,
    #[error("the parent block is missing for the required write")]
    MissingParent,
}

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
    /// hash has chain_length 1, its children have chain_length 2, and so on.
    /// Note that there is no block with chain_length 0 because there is no
    /// block with the zero hash.
    pub chain_length: u64,

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

pub struct BlockStoreBuilder {
    store_type: StoreType,
    busy_timeout: Option<u64>,
}

pub struct BlockStore {
    store_type: StoreType,
    // An optional connection to be always held open. This is a workaround to
    // prevent an in-memory storage from resetting, because it is getting reset
    // once the last open connection was removed.
    persistent_connection: Option<Connection>,
    busy_timeout: Option<u64>,
}

impl BlockStoreBuilder {
    pub fn memory() -> Self {
        BlockStoreBuilder {
            store_type: StoreType::Memory,
            busy_timeout: None,
        }
    }

    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        BlockStoreBuilder {
            store_type: StoreType::File(path.as_ref().to_path_buf()),
            busy_timeout: None,
        }
    }

    pub fn busy_timeout(self, busy_timeout: u64) -> Self {
        BlockStoreBuilder {
            busy_timeout: Some(busy_timeout),
            ..self
        }
    }

    pub fn build(self) -> BlockStore {
        BlockStore::init(self.store_type, self.busy_timeout)
    }
}

// persistent_connection does not implement Sync but is never actually used
// which makes it safe to be shared
unsafe impl Sync for BlockStore {}

pub struct BlockStoreConnection<B>
where
    B: Block,
{
    inner: Connection,
    dummy: std::marker::PhantomData<B>,
}

impl BlockStore {
    fn init(store_type: StoreType, busy_timeout: Option<u64>) -> Self {
        let mut store = Self {
            store_type,
            persistent_connection: None,
            busy_timeout,
        };

        let connection = store.connect_internal().unwrap();

        // TODO rename depth to chain_length (left it unchanged for compatibility)
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

                  create index if not exists ChainLengthIndex on BlockInfo(depth);

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
        let connection = match &self.store_type {
            StoreType::Memory => Connection::open("file::memory:?cache=shared"),
            StoreType::File(path) => Connection::open(path),
        }
        .map_err(|err| Error::BackendError(Box::new(err)))?;
        if let Some(busy_timeout) = self.busy_timeout {
            connection
                .busy_timeout(Duration::from_millis(busy_timeout))
                .map_err(|err| Error::BackendError(Box::new(err)))?;
        }
        Ok(connection)
    }

    pub fn connect<B>(&self) -> Result<BlockStoreConnection<B>, Error>
    where
        B: Block,
    {
        Ok(BlockStoreConnection {
            inner: self.connect_internal()?,
            dummy: std::marker::PhantomData,
        })
    }
}

impl<B> BlockStoreConnection<B>
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

        let tx = self
            .inner
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        if block_exists_internal(&tx, &block_hash)? {
            return Ok(());
        }

        let parent_hash = block.parent_id();

        // Always include a link to the parent.
        let mut back_links = vec![BackLink {
            distance: 1,
            block_hash: parent_hash.clone(),
        }];

        let chain_length = if parent_hash == B::Id::zero() {
            1
        } else {
            let parent_info = get_block_info_internal(&tx, &parent_hash).map_err(|e| match e {
                Error::BlockNotFound => Error::MissingParent,
                e => e,
            })?;
            assert!(parent_info.chain_length > 0);
            let chain_length = 1 + parent_info.chain_length;
            let fast_link = compute_fast_link(chain_length);
            let distance = chain_length - fast_link;
            if distance != 1 && fast_link > 0 {
                let far_block_info =
                    get_nth_ancestor_internal(&tx, &parent_hash, chain_length - 1 - fast_link)?;
                back_links.push(BackLink {
                    distance,
                    block_hash: far_block_info.block_hash,
                })
            }

            chain_length
        };

        let block_info = BlockInfo {
            block_hash: block_hash.clone(),
            chain_length,
            back_links,
        };

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
                Value::Integer(block_info.chain_length as i64),
                Value::Blob(parent.block_hash.serialize_as_vec().unwrap()),
                fast_distance,
                fast_hash,
            ])
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tx.commit()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Ok(())
    }

    pub fn get_block(&mut self, block_hash: &B::Id) -> Result<(B, BlockInfo<B::Id>), Error> {
        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let blk = tx
            .prepare_cached("select block from Blocks where hash = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[&block_hash.serialize_as_vec().unwrap()[..]], |row| {
                let x: Vec<u8> = row.get(0)?;
                Ok(B::deserialize(&x[..]).unwrap())
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?;

        let info = get_block_info_internal(&tx, block_hash)?;

        Ok((blk, info))
    }

    pub fn get_block_info(&mut self, block_hash: &B::Id) -> Result<BlockInfo<B::Id>, Error> {
        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        get_block_info_internal(&tx, block_hash)
    }

    pub fn get_blocks_by_chain_length(
        &mut self,
        chain_length: u64,
    ) -> Result<Vec<(B, BlockInfo<B::Id>)>, Error> {
        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let blocks = tx
            .prepare_cached(
                "select block from Blocks
                  inner join BlockInfo
                  on Blocks.hash = BlockInfo.hash
                  where BlockInfo.depth = ?",
            )
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_map(&[Value::from(chain_length as i64)], |row| {
                let x: Vec<u8> = row.get(0)?;
                Ok(B::deserialize(&x[..]).unwrap())
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::BackendError(Box::new(e)))?;

        let block_infos = Self::get_block_infos_by_chain_length_internal(&tx, chain_length)?;

        Ok(blocks.into_iter().zip(block_infos).collect())
    }

    pub fn get_block_infos_by_chain_length(
        &mut self,
        chain_length: u64,
    ) -> Result<Vec<BlockInfo<B::Id>>, Error> {
        Self::get_block_infos_by_chain_length_internal(&self.inner, chain_length)
    }

    fn get_block_infos_by_chain_length_internal(
        connection: &Connection,
        chain_length: u64,
    ) -> Result<Vec<BlockInfo<B::Id>>, Error> {
        connection
            .prepare_cached(
                "select hash, depth, parent, fast_distance, fast_hash from BlockInfo where depth = ?",
            )
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_map(&[Value::from(chain_length as i64)], |row| {
                let mut back_links = vec![BackLink {
                    distance: 1,
                    block_hash: blob_to_hash(row.get(2)?),
                }];

                let fast_distance: Option<i64> = row.get(3)?;
                if let Some(fast_distance) = fast_distance {
                    back_links.push(BackLink {
                        distance: fast_distance as u64,
                        block_hash: blob_to_hash(row.get(4)?),
                    });
                }

                let chain_length: i64 = row.get(1)?;

                Ok(BlockInfo {
                    block_hash: blob_to_hash(row.get(0)?),
                    chain_length: chain_length as u64,
                    back_links,
                })
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::BackendError(Box::new(e)))
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

    pub fn get_tag(&mut self, tag_name: &str) -> Result<Option<B::Id>, Error> {
        match self
            .inner
            .prepare_cached("select hash from Tags where name = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[&tag_name], |row| Ok(blob_to_hash(row.get(0)?)))
        {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(Error::BackendError(Box::new(err))),
        }
    }

    /// Check whether a block exists.
    pub fn block_exists(&mut self, block_hash: &B::Id) -> Result<bool, Error> {
        block_exists_internal(&self.inner, block_hash)
    }

    // Determine whether block 'ancestor' is an ancestor of block 'descendent'
    ///
    /// Returned values:
    /// - `Ok(Some(dist))` - `ancestor` is ancestor of `descendent`
    ///     and there are `dist` blocks between them
    /// - `Ok(None)` - `ancestor` is not ancestor of `descendent`
    /// - `Err(error)` - `ancestor` or `descendent` was not found
    pub fn is_ancestor(
        &mut self,
        ancestor: &B::Id,
        descendent: &B::Id,
    ) -> Result<Option<u64>, Error> {
        // Optimization.
        if ancestor == descendent {
            return Ok(Some(0));
        }

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let descendent = get_block_info_internal(&tx, descendent)?;

        if ancestor == &B::Id::zero() {
            return Ok(Some(descendent.chain_length));
        }

        let ancestor = get_block_info_internal(&tx, ancestor)?;

        // Bail out right away if the "descendent" does not have a
        // higher chain_length.
        if descendent.chain_length <= ancestor.chain_length {
            return Ok(None);
        }

        // Seek back from the descendent to check whether it has the
        // ancestor at the expected place.
        let info = get_nth_ancestor_internal(
            &tx,
            &descendent.block_hash,
            descendent.chain_length - ancestor.chain_length,
        )?;

        if info.block_hash == ancestor.block_hash {
            Ok(Some(descendent.chain_length - ancestor.chain_length))
        } else {
            Ok(None)
        }
    }

    pub fn get_nth_ancestor(
        &mut self,
        block_hash: &B::Id,
        distance: u64,
    ) -> Result<BlockInfo<B::Id>, Error> {
        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        get_nth_ancestor_internal(&tx, block_hash, distance)
    }
}

fn get_block_info_internal<Id>(
    connection: &Connection,
    block_hash: &Id,
) -> Result<BlockInfo<Id>, Error>
where
    Id: BlockId,
{
    connection
        .prepare_cached(
            "select depth, parent, fast_distance, fast_hash from BlockInfo where hash = ?",
        )
        .map_err(|err| Error::BackendError(Box::new(err)))?
        .query_row(&[&block_hash.serialize_as_vec().unwrap()[..]], |row| {
            let mut back_links = vec![BackLink {
                distance: 1,
                block_hash: blob_to_hash(row.get(1)?),
            }];

            let fast_distance: Option<i64> = row.get(2)?;
            if let Some(fast_distance) = fast_distance {
                back_links.push(BackLink {
                    distance: fast_distance as u64,
                    block_hash: blob_to_hash(row.get(3)?),
                });
            }

            let chain_length: i64 = row.get(0)?;

            Ok(BlockInfo {
                block_hash: block_hash.clone(),
                chain_length: chain_length as u64,
                back_links,
            })
        })
        .map_err(|err| match err {
            rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
            err => Error::BackendError(Box::new(err)),
        })
}

fn block_exists_internal<Id>(connection: &Connection, block_hash: &Id) -> Result<bool, Error>
where
    Id: BlockId,
{
    match get_block_info_internal(connection, block_hash) {
        Ok(_) => Ok(true),
        Err(Error::BlockNotFound) => Ok(false),
        Err(err) => Err(err),
    }
}

fn get_nth_ancestor_internal<Id>(
    connection: &Connection,
    block_hash: &Id,
    distance: u64,
) -> Result<BlockInfo<Id>, Error>
where
    Id: BlockId,
{
    for_path_to_nth_ancestor_internal(connection, block_hash, distance, |_| {})
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
    store: &mut BlockStoreConnection<B>,
    block_hash: &B::Id,
    distance: u64,
    callback: F,
) -> Result<BlockInfo<B::Id>, Error>
where
    B: Block,
    F: FnMut(&BlockInfo<B::Id>),
{
    let tx = store
        .inner
        .transaction()
        .map_err(|err| Error::BackendError(Box::new(err)))?;

    for_path_to_nth_ancestor_internal(&tx, block_hash, distance, callback)
}

fn for_path_to_nth_ancestor_internal<Id, F>(
    connection: &Connection,
    block_hash: &Id,
    distance: u64,
    mut callback: F,
) -> Result<BlockInfo<Id>, Error>
where
    Id: BlockId,
    F: FnMut(&BlockInfo<Id>),
{
    let mut cur_block_info = get_block_info_internal(connection, block_hash)?;

    if distance >= cur_block_info.chain_length {
        // FIXME: return error
        panic!(
            "distance {} > chain length {}",
            distance, cur_block_info.chain_length
        );
    }

    let target = cur_block_info.chain_length - distance;

    // Travel back through the chain using the back links until we
    // reach the desired block.
    while target < cur_block_info.chain_length {
        // We're not there yet. Use the back link that takes us
        // furthest back in the chain, without going beyond the
        // block we're looking for.
        let best_link = cur_block_info
            .back_links
            .iter()
            .filter(|x| cur_block_info.chain_length - target >= x.distance)
            .max_by_key(|x| x.distance)
            .unwrap()
            .clone();
        callback(&cur_block_info);
        cur_block_info = get_block_info_internal(connection, &best_link.block_hash)?;
    }

    assert_eq!(target, cur_block_info.chain_length);

    Ok(cur_block_info)
}

/// Compute the fast link for a block with a given chain_length. Successive
/// blocks make a chain_length jump equal to differents powers of two, minus
/// 1, e.g. 1, 3, 7, 15, 31, ...
fn compute_fast_link(chain_length: u64) -> u64 {
    let order = chain_length % 32;
    let distance = if order == 0 { 1 } else { (1 << order) - 1 };
    if distance < chain_length {
        chain_length - distance
    } else {
        0
    }
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
        chain_length: ChainLength,
        data: Box<[u8]>,
    }

    impl Block {
        pub fn genesis(data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: BlockId::zero(),
                date: BlockDate::from_epoch_slot_id(0, 0),
                chain_length: ChainLength(1),
                data: data.unwrap_or_default(),
            }
        }

        pub fn make_child(&self, data: Option<Box<[u8]>>) -> Self {
            Self {
                id: BlockId::generate(),
                parent: self.id,
                date: BlockDate::from_epoch_slot_id(self.date.0, self.date.1 + 1),
                chain_length: ChainLength(self.chain_length.0 + 1),
                data: data.unwrap_or_default(),
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
                chain_length: ChainLength(codec.get_u64()?),
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
    use super::*;
    use chain_core::property::{Block as _, BlockId as _};
    use rand_core::{OsRng, RngCore};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a Vec<A>) -> &'a A {
        let s = rng.next_u32() as usize;
        // this doesn't need to be uniform
        &v[s % v.len()]
    }

    pub fn generate_chain<R: RngCore>(
        rng: &mut R,
        store: &mut BlockStoreConnection<Block>,
    ) -> Vec<Block> {
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
        let mut store = BlockStoreBuilder::file("file:test_put_get?mode=memory&cache=shared")
            .build()
            .connect()
            .unwrap();
        assert!(store.get_tag("tip").unwrap().is_none());

        match store.put_tag("tip", &BlockId::zero()) {
            Err(Error::BlockNotFound) => {}
            err => panic!(err),
        }

        let genesis_block = Block::genesis(None);
        store.put_block(&genesis_block).unwrap();
        let (genesis_block_restored, block_info) = store.get_block(&genesis_block.id()).unwrap();
        assert_eq!(genesis_block, genesis_block_restored);
        assert_eq!(block_info.block_hash, genesis_block.id());
        assert_eq!(block_info.chain_length, genesis_block.chain_length().0);
        assert_eq!(block_info.back_links.len(), 1);
        assert_eq!(block_info.parent_id(), BlockId::zero());

        let blocks = store.get_blocks_by_chain_length(1).unwrap();
        println!("{:?}", blocks);
        assert_eq!(genesis_block, blocks[0].0);
        assert_eq!(blocks[0].1.block_hash, genesis_block.id());
        assert_eq!(blocks[0].1.chain_length, genesis_block.chain_length().0);
        assert_eq!(blocks[0].1.back_links.len(), 1);
        assert_eq!(blocks[0].1.parent_id(), BlockId::zero());

        store.put_tag("tip", &genesis_block.id()).unwrap();
        assert_eq!(store.get_tag("tip").unwrap().unwrap(), genesis_block.id());
    }

    #[test]
    pub fn test_nth_ancestor() {
        let mut rng = OsRng;
        let mut store = BlockStoreBuilder::file("file:test_nth_ancestor?mode=memory&cache=shared")
            .build()
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

            let ancestor_info = for_path_to_nth_ancestor(&mut store, &block.id(), distance, |_| {
                blocks_fetched += 1;
            })
            .unwrap();

            assert_eq!(
                ancestor_info.chain_length + distance,
                block.chain_length().0
            );

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
    fn simultaneous_read_write() {
        let mut rng = OsRng;
        let store =
            BlockStoreBuilder::file("file:simultaneous_read_write?mode=memory&cache=shared")
                .build();

        let mut conn = store.connect().unwrap();

        let genesis_block = Block::genesis(None);
        conn.put_block(&genesis_block).unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            blocks.push(block.clone());
            conn.put_block(&block).unwrap()
        }

        let mut conn_1 = store.connect().unwrap();
        let blocks_1 = blocks.clone();

        let thread_1 = std::thread::spawn(move || {
            for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
                let block_id = blocks_1
                    .get(rng.next_u32() as usize % blocks_1.len())
                    .unwrap()
                    .id();
                let _: (Block, _) = conn_1.get_block(&block_id).unwrap();
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
