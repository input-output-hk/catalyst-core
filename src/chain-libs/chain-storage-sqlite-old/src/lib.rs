use chain_core::property::{Block, BlockId, Serialize};
use chain_storage::{
    error::Error,
    store::{BackLink, BlockInfo, BlockStore},
};
use rusqlite::{types::Value, Connection};
use std::path::{Path, PathBuf};

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

        if let StoreType::Memory = store.store_type {
            store.persistent_connection = Some(connection);
        }

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

fn blob_to_hash<Id: BlockId>(blob: Vec<u8>) -> Id {
    Id::deserialize(&blob[..]).unwrap()
}

impl<B> BlockStore for SQLiteBlockStoreConnection<B>
where
    B: Block,
{
    type Block = B;

    fn put_block_internal(&mut self, block: &B, block_info: BlockInfo<B::Id>) -> Result<(), Error> {
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

    fn get_block(&self, block_hash: &B::Id) -> Result<(B, BlockInfo<B::Id>), Error> {
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

    fn get_block_info(&self, block_hash: &B::Id) -> Result<BlockInfo<B::Id>, Error> {
        self.inner
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

    fn put_tag(&mut self, tag_name: &str, block_hash: &B::Id) -> Result<(), Error> {
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

    fn get_tag(&self, tag_name: &str) -> Result<Option<B::Id>, Error> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_storage::store::testing::Block as TestBlock;
    use rand_core::{OsRng, RngCore};

    const SIMULTANEOUS_READ_WRITE_ITERS: usize = 50;

    #[test]
    fn put_get() {
        let mut store = SQLiteBlockStore::file("file:test_put_get?mode=memory&cache=shared")
            .connect()
            .unwrap();
        chain_storage::store::testing::test_put_get(&mut store);
    }

    #[test]
    fn nth_ancestor() {
        let mut rng = OsRng;
        let mut store = SQLiteBlockStore::file("file:test_nth_ancestor?mode=memory&cache=shared")
            .connect()
            .unwrap();
        chain_storage::store::testing::test_nth_ancestor(&mut rng, &mut store);
    }

    #[test]
    fn iterate_range() {
        let mut rng = OsRng;
        let mut store = SQLiteBlockStore::file("file:test_iterate_range?mode=memory&cache=shared")
            .connect()
            .unwrap();
        chain_storage::store::testing::test_iterate_range(&mut rng, &mut store);
    }

    #[test]
    fn simultaneous_read_write() {
        let mut rng = OsRng;
        let mut store =
            SQLiteBlockStore::file("file:test_simultaneous_read_write?mode=memory&cache=shared");

        let mut conn = store.connect::<TestBlock>().unwrap();

        let genesis_block = TestBlock::genesis(None);
        conn.put_block(&genesis_block).unwrap();
        let mut blocks = vec![genesis_block];

        for _ in 1..SIMULTANEOUS_READ_WRITE_ITERS {
            let last_block = blocks.get(rng.next_u32() as usize % blocks.len()).unwrap();
            let block = last_block.make_child(None);
            blocks.push(block.clone());
            conn.put_block(&block).unwrap()
        }

        let conn_1 = store.connect::<TestBlock>().unwrap();
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
