use crate::data::{Block, BlockId, BlockIds, Header};
use crate::error::Error;
use async_trait::async_trait;
use futures::prelude::*;

/// Client-side interface for the remote blockchain node service
/// responsible for providing access to blocks.
#[async_trait]
pub trait BlockService {
    /// Requests the identifier of the genesis block from the service node.
    ///
    /// The implementation can also perform version information checks to
    /// ascertain that the client use compatible protocol versions.
    ///
    /// This method should be called first after establishing the client
    /// connection.
    async fn handshake(&mut self) -> Result<BlockId, HandshakeError>;

    /// Requests the header of the tip block in the node's chain.
    async fn tip(&mut self) -> Result<Header, Error>;

    /// The type of an asynchronous stream that provides blocks in
    /// response to method `get_blocks`.
    type GetBlocksStream: Stream<Item = Result<Block, Error>>;

    async fn get_blocks(&mut self, ids: BlockIds) -> Result<Self::GetBlocksStream, Error>;
}

/// An error that the future returned by `BlockService::handshake` can
/// resolve to.
#[derive(Debug, thiserror::Error)]
pub enum HandshakeError {
    /// Error occurred with the protocol request.
    #[error("{0}")]
    Rpc(#[source] Error),
    /// The protocol version reported by the server is not supported.
    /// Carries the reported version in a human-readable form.
    #[error("unsupported protocol version {0}")]
    UnsupportedVersion(Box<str>),
    #[error("invalid genesis block payload")]
    InvalidBlock0(#[source] Error),
}
