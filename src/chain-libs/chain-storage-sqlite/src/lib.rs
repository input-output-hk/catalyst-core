mod index;

use chain_core::property::{Block, BlockId, Serialize};
use chain_storage::{
    error::Error,
    store::{BackLink, BlockInfo, BlockStore},
};
use index::DbIndex;
use rusqlite::types::Value;
use std::path::Path;
use thiserror::Error;

type RowId = isize;

#[derive(Debug, Error)]
enum IndexError {
    #[error("this block already exists")]
    BlockExists,
    #[error("could not find block")]
    BlockNotFound,
}

#[derive(Debug, Error)]
enum IndexCreationError {
    #[error("cannot insert to index: {0}")]
    IndexError(IndexError),
    #[error("cannot read from sqlite: {0}")]
    SQLiteError(rusqlite::Error),
}

impl std::convert::From<rusqlite::Error> for IndexCreationError {
    fn from(e: rusqlite::Error) -> Self {
        IndexCreationError::SQLiteError(e)
    }
}

type IndexResult = Result<(), IndexError>;

#[derive(Clone)]
struct ChainStorageIndex<B>
where
    B: Block,
{
    blocks_index: DbIndex<B::Id, RowId>,
    block_info_index: DbIndex<B::Id, RowId>,
    tags_index: DbIndex<String, RowId>,
}

impl<B> ChainStorageIndex<B>
where
    B: Block,
{
    pub fn new() -> Self {
        Self {
            blocks_index: DbIndex::new(),
            block_info_index: DbIndex::new(),
            tags_index: DbIndex::new(),
        }
    }

    pub fn get_block(&self, key: &B::Id) -> Option<&RowId> {
        self.blocks_index.get(key)
    }

    pub fn add_block_check(&self, block_id: &B::Id) -> IndexResult {
        if self.blocks_index.get(block_id).is_some()
            || self.block_info_index.get(block_id).is_some()
        {
            return Err(IndexError::BlockExists);
        }
        Ok(())
    }

    pub fn add_block(&mut self, block_id: B::Id, row_id: RowId) -> IndexResult {
        self.add_block_check(&block_id)?;
        self.blocks_index.add(block_id, row_id);
        Ok(())
    }

    pub fn get_block_info(&self, key: &B::Id) -> Option<&RowId> {
        self.block_info_index.get(key)
    }

    pub fn add_block_info(&mut self, block_id: B::Id, row_id: RowId) -> IndexResult {
        if self.get_block(&block_id).is_none() {
            return Err(IndexError::BlockNotFound);
        }
        self.block_info_index.add(block_id, row_id);
        Ok(())
    }

    pub fn get_tag(&self, tag: &String) -> Option<&RowId> {
        self.tags_index.get(tag)
    }

    pub fn add_tag_check(&self, block_id: &B::Id) -> IndexResult {
        if self.get_block(block_id).is_none() {
            return Err(IndexError::BlockNotFound);
        }
        Ok(())
    }

    pub fn add_tag(&mut self, tag: String, block_id: &B::Id, row_id: RowId) -> IndexResult {
        self.add_tag_check(block_id)?;
        self.tags_index.add(tag, row_id);
        Ok(())
    }
}

#[derive(Clone)]
pub struct SQLiteBlockStore<B>
where
    B: Block,
{
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
    dummy: std::marker::PhantomData<B>,
    index: ChainStorageIndex<B>,
}

impl<B> SQLiteBlockStore<B>
where
    B: Block,
{
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path);
        let pool = r2d2::Pool::new(manager).unwrap();

        let connection = pool.get().unwrap();

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

        /*
        connection
            .execute("pragma synchronous = off", rusqlite::NO_PARAMS)
            .unwrap();
        */

        connection
            .execute_batch("pragma journal_mode = WAL")
            .unwrap();

        let mut index = ChainStorageIndex::new();
        connection
            .prepare("select rowid, hash from Blocks")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0);
                let hash = blob_to_hash(row.get(1));

                index
                    .add_block(hash, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
            .unwrap();

        connection
            .prepare("select rowid, hash from BlockInfo")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0);
                let hash = blob_to_hash(row.get(1));

                index
                    .add_block_info(hash, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
            .unwrap();

        connection
            .prepare("select rowid, hash, name from Tags")
            .unwrap()
            .query_and_then(rusqlite::NO_PARAMS, |row| {
                let row_id = row.get(0);
                let hash = blob_to_hash(row.get(1));
                let name = row.get(2);

                index
                    .add_tag(name, &hash, row_id)
                    .map_err(IndexCreationError::IndexError)
            })
            .unwrap()
            .try_for_each(std::convert::identity)
            .unwrap();

        SQLiteBlockStore {
            pool,
            dummy: std::marker::PhantomData,
            index,
        }
    }
}

fn blob_to_hash<Id: BlockId>(blob: Vec<u8>) -> Id {
    Id::deserialize(&blob[..]).unwrap()
}

impl<B> BlockStore for SQLiteBlockStore<B>
where
    B: Block,
{
    type Block = B;

    fn put_block_internal(&mut self, block: &B, block_info: BlockInfo<B::Id>) -> Result<(), Error> {
        self.index
            .add_block_check(&block_info.block_hash)
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let mut conn = self
            .pool
            .get()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let tx = conn
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

        let block_row_id: RowId = tx
            .prepare_cached("select last_insert_rowid()")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(rusqlite::NO_PARAMS, |row| row.get(0))
            .map_err(|err| Error::BackendError(Box::new(err)))?;

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

        let block_info_row_id: RowId = tx
            .prepare_cached("select last_insert_rowid()")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(rusqlite::NO_PARAMS, |row| row.get(0))
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        tx.commit()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        self.index
            .add_block(block_info.block_hash.clone(), block_row_id)
            .unwrap();
        self.index
            .add_block_info(block_info.block_hash.clone(), block_info_row_id)
            .unwrap();

        Ok(())
    }

    fn get_block(&self, block_hash: &B::Id) -> Result<(B, BlockInfo<B::Id>), Error> {
        let row_id = self
            .index
            .get_block(block_hash)
            .ok_or(Error::BlockNotFound)?;

        let blk = self
            .pool
            .get()
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .prepare_cached("select block from Blocks where rowid = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[row_id], |row| {
                let x: Vec<u8> = row.get(0);
                B::deserialize(&x[..]).unwrap()
            })
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        let info = self.get_block_info(block_hash)?;

        Ok((blk, info))
    }

    fn get_block_info(&self, block_hash: &B::Id) -> Result<BlockInfo<B::Id>, Error> {
        let row_id = self
            .index
            .get_block_info(block_hash)
            .ok_or(Error::BlockNotFound)?;

        self.pool
            .get()
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .prepare_cached(
                "select depth, parent, fast_distance, fast_hash from BlockInfo where rowid = ?",
            )
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[row_id], |row| {
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
            .map_err(|err| Error::BackendError(Box::new(err)))
    }

    fn put_tag(&mut self, tag_name: &str, block_hash: &B::Id) -> Result<(), Error> {
        self.index
            .add_tag_check(block_hash)
            .map_err(|err| match err {
                IndexError::BlockNotFound => Error::BlockNotFound,
                err => Error::BackendError(Box::new(err)),
            })?;

        let conn = self
            .pool
            .get()
            .map_err(|err| Error::BackendError(Box::new(err)))?;

        match self.index.get_tag(&tag_name.to_owned()) {
            Some(row_id) => conn
                .prepare_cached("replace into Tags (rowid, name, hash) values(?, ?, ?)")
                .map_err(|err| Error::BackendError(Box::new(err)))?
                .execute(&[
                    Value::Integer(*row_id as i64),
                    Value::Text(tag_name.to_string()),
                    Value::Blob(block_hash.serialize_as_vec().unwrap()),
                ]),
            None => conn
                .prepare_cached("insert into Tags (name, hash) values(?, ?)")
                .map_err(|err| Error::BackendError(Box::new(err)))?
                .execute(&[
                    Value::Text(tag_name.to_string()),
                    Value::Blob(block_hash.serialize_as_vec().unwrap()),
                ]),
        }
        .map_err(|err| Error::BackendError(Box::new(err)))?;

        conn.prepare_cached("select last_insert_rowid()")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(rusqlite::NO_PARAMS, |row| {
                self.index
                    .add_tag(tag_name.to_owned(), block_hash, row.get(0))
                    .map_err(|err| Error::BackendError(Box::new(err)))
            })
            .map_err(|err| Error::BackendError(Box::new(err)))??;

        Ok(())
    }

    fn get_tag(&self, tag_name: &str) -> Result<Option<B::Id>, Error> {
        let row_id = match self.index.get_tag(&tag_name.to_owned()) {
            Some(v) => v,
            None => return Ok(None),
        };

        match self
            .pool
            .get()
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .prepare_cached("select hash from Tags where rowid = ?")
            .map_err(|err| Error::BackendError(Box::new(err)))?
            .query_row(&[row_id], |row| blob_to_hash(row.get(0)))
        {
            Ok(s) => Ok(Some(s)),
            Err(err) => Err(Error::BackendError(Box::new(err))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chain_storage::store::testing::Block;
    use rand_os::OsRng;

    #[test]
    pub fn put_get() {
        let mut store = SQLiteBlockStore::<Block>::new(":memory:");
        chain_storage::store::testing::test_put_get(&mut store);
    }

    #[test]
    pub fn nth_ancestor() {
        let mut rng = OsRng::new().unwrap();
        let mut store = SQLiteBlockStore::<Block>::new(":memory:");
        chain_storage::store::testing::test_nth_ancestor(&mut rng, &mut store);
    }

    #[test]
    pub fn iterate_range() {
        let mut rng = OsRng::new().unwrap();
        let mut store = SQLiteBlockStore::<Block>::new(":memory:");
        chain_storage::store::testing::test_iterate_range(&mut rng, &mut store);
    }
}
