use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to open the database directory")]
    Open(#[source] std::io::Error),
    #[error("block not found")]
    BlockNotFound,
    #[error("database backend error")]
    VolatileBackendError(#[from] sled::Error),
    #[error("permanent store error")]
    PermanentBackendError(#[from] data_pile::Error),
    #[error("Block already present in DB")]
    BlockAlreadyPresent,
    #[error("the parent block is missing for the required write")]
    MissingParent,
    #[error("branch with the requested tip does not exist")]
    BranchNotFound,
    #[error("failed to serialize block metadata")]
    BlockInfoSerialize(#[source] std::io::Error),
    #[error("failed to deserialize block metadata")]
    BlockInfoDeserialize(#[source] std::io::Error),
    #[error("the database is consistent")]
    Inconsistent(#[from] ConsistencyFailure),
    #[error(
        "cannot iterate over blocks because the provided distance is bigger than the chain length"
    )]
    CannotIterate,
}

#[derive(Debug, Error)]
pub enum ConsistencyFailure {
    #[error("stored block is missing its parent")]
    MissingParentBlock,
    #[error("block is listed in the chain length index but is not stored")]
    ChainLength,
    #[error("BlockInfo is present but the corresponding block is not stored")]
    BlockInfo,
    #[error("tagged block is not stored")]
    TaggedBlock,
    #[error("expected to see a block in the permanent storage")]
    MissingPermanentBlock,
}
