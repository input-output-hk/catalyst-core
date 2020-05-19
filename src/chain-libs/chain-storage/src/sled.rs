use chain_core::property::{Block, BlockId, Deserialize, Serialize};
use sled::{ConflictableTransactionError, TransactionError, Transactional};
use std::{marker::PhantomData, path::PathBuf};
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

pub struct BlockStore<B> {
    inner: sled::Db,
    dummy: PhantomData<B>,
}

pub enum StoreType {
    Memory,
    File(PathBuf),
}

impl<B> BlockStore<B>
where
    B: Block,
{
    pub fn new(store_type: StoreType) -> Result<Self, Error> {
        let inner = match store_type {
            StoreType::Memory => todo!("in-memory storage is not implemented yet"),
            StoreType::File(filename) => {
                sled::open(filename).map_err(|e| Error::BackendError(Box::new(e)))
            }
        }?;

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

        blocks
            .transaction(move |blocks| {
                let block_hash = block.id().serialize_as_vec().unwrap();

                if blocks.get(block_hash.clone())?.is_some() {
                    return Err(ConflictableTransactionError::Abort(
                        Error::BlockAlreadyPresent,
                    ));
                }

                if block.parent_id() != B::Id::zero() {
                    let parent_id = block.parent_id().serialize_as_vec().unwrap();
                    if blocks.get(parent_id)?.is_none() {
                        return Err(ConflictableTransactionError::Abort(Error::MissingParent));
                    }
                }

                blocks.insert(block_hash, block.serialize_as_vec().unwrap())?;

                Ok(())
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => Error::BackendError(Box::new(err)),
            })
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
}
