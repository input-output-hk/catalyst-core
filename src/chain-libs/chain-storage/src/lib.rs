use crate::index::{ChainStorageIndex, IndexCreationError};
use chain_core::property::{Block, BlockId, Serialize};
use rusqlite::{types::Value, Connection, TransactionBehavior};
use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
    time::Duration,
};
use thiserror::Error;

mod index;
pub mod sled;

#[derive(Debug, Error)]
pub enum Error {
    #[error("block not found")]
    BlockNotFound,
    // FIXME: add BlockId
    #[error("database backend error")]
    BackendError(#[from] Box<dyn std::error::Error + Send + Sync>),
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

pub struct BlockStoreBuilder<B> {
    store_type: StoreType,
    busy_timeout: Option<u64>,
    dummy: std::marker::PhantomData<B>,
}

enum StoreType {
    Memory,
    File(PathBuf),
}

pub struct BlockStore<B>
where
    B: Block,
{
    store_type: StoreType,
    // An optional connection to be always held open. This is a workaround to
    // prevent an in-memory storage from resetting, because it is getting reset
    // once the last open connection was removed.
    persistent_connection: Option<Connection>,
    index: Arc<RwLock<ChainStorageIndex<B>>>,
    busy_timeout: Option<u64>,
}

// persistent_connection does not implement Sync but is never actually used
// which makes it safe to be shared
unsafe impl<B> Sync for BlockStore<B> where B: Block {}

pub struct BlockStoreConnection<B>
where
    B: Block,
{
    inner: Connection,
    index: Arc<RwLock<ChainStorageIndex<B>>>,
}

impl<B> BlockStoreBuilder<B>
where
    B: Block,
{
    pub fn memory() -> Self {
        BlockStoreBuilder {
            store_type: StoreType::Memory,
            busy_timeout: None,
            dummy: PhantomData,
        }
    }

    pub fn file<P: AsRef<Path>>(path: P) -> Self {
        BlockStoreBuilder {
            store_type: StoreType::File(path.as_ref().to_path_buf()),
            busy_timeout: None,
            dummy: PhantomData,
        }
    }

    pub fn busy_timeout(self, busy_timeout: u64) -> Self {
        BlockStoreBuilder {
            busy_timeout: Some(busy_timeout),
            ..self
        }
    }

    pub fn build(self) -> BlockStore<B> {
        BlockStore::init(self.store_type, self.busy_timeout)
    }
}

impl<B> BlockStore<B>
where
    B: Block,
{
    fn init(store_type: StoreType, busy_timeout: Option<u64>) -> Self {
        let index = Arc::new(RwLock::new(ChainStorageIndex::new()));

        let mut store = Self {
            store_type,
            persistent_connection: None,
            index: index.clone(),
            busy_timeout,
        };

        let connection = store.connect_internal().unwrap();
        let mut index = index.write().unwrap();

        // TODO rename depth to chain_length (left it unchanged for compatibility)
        connection
            .execute_batch(
                r#"
                  begin;

                  create table if not exists BlockInfo (
                    hash blob not null,
                    depth integer not null,
                    parent blob not null,
                    fast_distance blob,
                    fast_hash blob
                  );

                  create table if not exists Blocks (
                    hash blob not null,
                    block blob not null
                  );

                  create table if not exists Tags (
                    name text not null,
                    hash blob not null
                  );

                  commit;
                "#,
            )
            .unwrap();

        connection
            .execute_batch("pragma journal_mode = WAL")
            .unwrap();

        connection
            .prepare("select rowid, hash from Blocks")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0).unwrap();
                let hash = blob_to_hash(row.get(1).unwrap());

                index
                    .add_block(hash, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
            .unwrap();

        connection
            .prepare("select rowid, hash, depth from BlockInfo")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0).unwrap();
                let hash: B::Id = blob_to_hash(row.get(1).unwrap());
                let chain_length: i64 = row.get(2).unwrap();

                index
                    .add_block_info(hash, chain_length as u64, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
            .unwrap();

        connection
            .prepare("select rowid, hash, name from Tags")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0).unwrap();
                let hash = blob_to_hash(row.get(1).unwrap());
                let name = row.get(2).unwrap();

                index
                    .add_tag(name, &hash, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
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

    pub fn connect(&self) -> Result<BlockStoreConnection<B>, Error> {
        Ok(BlockStoreConnection {
            inner: self.connect_internal()?,
            index: self.index.clone(),
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

        let mut index = self.index.write().unwrap();

        let tx = self
            .inner
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        if Self::block_exists_internal(&tx, &index, &block_hash)? {
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
            let parent_info =
                Self::get_block_info_internal(&tx, &index, &parent_hash).map_err(|e| match e {
                    Error::BlockNotFound => Error::MissingParent,
                    e => e,
                })?;
            assert!(parent_info.chain_length > 0);
            let chain_length = 1 + parent_info.chain_length;
            let fast_link = compute_fast_link(chain_length);
            let distance = chain_length - fast_link;
            if distance != 1 && fast_link > 0 {
                let far_block_info = Self::get_nth_ancestor_internal(
                    &tx,
                    &index,
                    &parent_hash,
                    chain_length - 1 - fast_link,
                )?;
                back_links.push(BackLink {
                    distance,
                    block_hash: far_block_info.block_hash,
                })
            }

            chain_length
        };

        let block_info = BlockInfo {
            block_hash,
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

        let block_row_id = tx.last_insert_rowid();

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

        let block_info_row_id = tx.last_insert_rowid();

        index
            .add_block(block_info.block_hash.clone(), block_row_id as isize)
            .unwrap();

        index
            .add_block_info(
                block_info.block_hash.clone(),
                block_info.chain_length,
                block_info_row_id as isize,
            )
            .unwrap();

        tx.commit()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Ok(())
    }

    pub fn get_block(&mut self, block_hash: &B::Id) -> Result<(B, BlockInfo<B::Id>), Error> {
        let index = self.index.read().unwrap();

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Self::get_block_internal(&tx, &index, block_hash)
    }

    fn get_block_internal(
        connection: &Connection,
        index: &ChainStorageIndex<B>,
        block_hash: &B::Id,
    ) -> Result<(B, BlockInfo<B::Id>), Error> {
        let row_id = index.get_block(block_hash).ok_or(Error::BlockNotFound)?;

        let blk = connection
            .prepare_cached("select block from Blocks where rowid = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[Value::Integer(*row_id as i64)], |row| {
                let x: Vec<u8> = row.get(0)?;
                Ok(B::deserialize(&x[..]).unwrap())
            })
            .map_err(|err| match err {
                rusqlite::Error::QueryReturnedNoRows => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?;

        let info = Self::get_block_info_internal(connection, &index, block_hash)?;

        Ok((blk, info))
    }

    fn do_by_chain_length<T, F>(&mut self, chain_length: u64, f: F) -> Result<Vec<T>, Error>
    where
        F: Fn(&Connection, &ChainStorageIndex<B>, &B::Id) -> Result<T, Error>,
    {
        let index = self.index.read().unwrap();

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let hashes = match index.get_block_by_chain_length(chain_length) {
            Some(hashes) => hashes,
            None => return Ok(Vec::new()),
        };

        hashes
            .iter()
            .map(|block_hash| f(&tx, &index, block_hash))
            .collect()
    }

    #[allow(clippy::type_complexity)]
    pub fn get_blocks_by_chain_length(
        &mut self,
        chain_length: u64,
    ) -> Result<Vec<(B, BlockInfo<B::Id>)>, Error> {
        self.do_by_chain_length(chain_length, Self::get_block_internal)
    }

    pub fn get_block_info(&mut self, block_hash: &B::Id) -> Result<BlockInfo<B::Id>, Error> {
        let index = self.index.read().unwrap();

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Self::get_block_info_internal(&tx, &index, block_hash)
    }

    pub fn get_block_infos_by_chain_length(
        &mut self,
        chain_length: u64,
    ) -> Result<Vec<BlockInfo<B::Id>>, Error> {
        self.do_by_chain_length(chain_length, Self::get_block_info_internal)
    }

    fn get_block_info_internal(
        connection: &Connection,
        index: &ChainStorageIndex<B>,
        block_hash: &B::Id,
    ) -> Result<BlockInfo<B::Id>, Error> {
        let row_id = index
            .get_block_info(block_hash)
            .ok_or(Error::BlockNotFound)?;

        connection
            .prepare_cached(
                "select depth, parent, fast_distance, fast_hash from BlockInfo where rowid = ?",
            )
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[Value::Integer(*row_id as i64)], |row| {
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

    pub fn put_tag(&mut self, tag_name: &str, block_hash: &B::Id) -> Result<(), Error> {
        let mut index = self.index.write().unwrap();
        let tx = self
            .inner
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        match index.get_tag(&tag_name.to_owned()) {
            Some(row_id) => tx
                .prepare_cached("replace into Tags (rowid, name, hash) values(?, ?, ?)")
                .map_err(|err| Error::BackendError(Box::new(err)))?
                .execute(&[
                    Value::Integer(*row_id as i64),
                    Value::Text(tag_name.to_string()),
                    Value::Blob(block_hash.serialize_as_vec().unwrap()),
                ]),
            None => {
                if index.get_block(block_hash).is_none() {
                    return Err(Error::BlockNotFound);
                }

                tx.prepare_cached("insert into Tags (name, hash) values(?, ?)")
                    .map_err(|err| Error::BackendError(Box::new(err)))?
                    .execute(&[
                        Value::Text(tag_name.to_string()),
                        Value::Blob(block_hash.serialize_as_vec().unwrap()),
                    ])
            }
        }
        .map_err(|err| Error::BackendError(Box::new(err)))?;

        let row_id = tx.last_insert_rowid();
        index
            .add_tag(tag_name.to_owned(), block_hash, row_id as isize)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tx.commit()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Ok(())
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
        let index = self.index.read().unwrap();
        Self::block_exists_internal(&self.inner, &index, block_hash)
    }

    fn block_exists_internal(
        connection: &Connection,
        index: &ChainStorageIndex<B>,
        block_hash: &B::Id,
    ) -> Result<bool, Error> {
        match Self::get_block_info_internal(connection, index, block_hash) {
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
    pub fn is_ancestor(
        &mut self,
        ancestor: &B::Id,
        descendent: &B::Id,
    ) -> Result<Option<u64>, Error> {
        // Optimization.
        if ancestor == descendent {
            return Ok(Some(0));
        }

        let index = self.index.read().unwrap();

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let descendent = Self::get_block_info_internal(&tx, &index, &descendent)?;

        if ancestor == &B::Id::zero() {
            return Ok(Some(descendent.chain_length));
        }

        let ancestor = Self::get_block_info_internal(&tx, &index, &ancestor)?;

        // Bail out right away if the "descendent" does not have a
        // higher chain_length.
        if descendent.chain_length <= ancestor.chain_length {
            return Ok(None);
        }

        // Seek back from the descendent to check whether it has the
        // ancestor at the expected place.
        let info = Self::get_nth_ancestor_internal(
            &tx,
            &index,
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
        let index = self.index.read().unwrap();

        let tx = self
            .inner
            .transaction()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        Self::get_nth_ancestor_internal(&tx, &index, block_hash, distance)
    }

    fn get_nth_ancestor_internal(
        connection: &Connection,
        index: &ChainStorageIndex<B>,
        block_hash: &B::Id,
        distance: u64,
    ) -> Result<BlockInfo<B::Id>, Error> {
        for_path_to_nth_ancestor_internal::<B, _>(connection, index, block_hash, distance, |_| {})
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
    store: &mut BlockStoreConnection<B>,
    block_hash: &B::Id,
    distance: u64,
    callback: F,
) -> Result<BlockInfo<B::Id>, Error>
where
    B: Block,
    F: FnMut(&BlockInfo<B::Id>),
{
    let index = store.index.read().unwrap();

    let tx = store
        .inner
        .transaction()
        .map_err(|err| Error::BackendError(Box::new(err)))?;

    for_path_to_nth_ancestor_internal::<B, F>(&tx, &index, block_hash, distance, callback)
}

fn for_path_to_nth_ancestor_internal<B, F>(
    connection: &Connection,
    index: &ChainStorageIndex<B>,
    block_hash: &B::Id,
    distance: u64,
    mut callback: F,
) -> Result<BlockInfo<B::Id>, Error>
where
    B: Block,
    F: FnMut(&BlockInfo<B::Id>),
{
    let mut cur_block_info =
        BlockStoreConnection::<B>::get_block_info_internal(connection, index, block_hash)?;

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
        cur_block_info = BlockStoreConnection::<B>::get_block_info_internal(
            connection,
            index,
            &best_link.block_hash,
        )?;
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

    #[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Copy)]
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

    pub fn pick_from_vector<'a, A, R: RngCore>(rng: &mut R, v: &'a [A]) -> &'a A {
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
            BlockStoreBuilder::file("file:test_simultaneous_read_write?mode=memory&cache=shared")
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
