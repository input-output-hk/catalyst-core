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
}
