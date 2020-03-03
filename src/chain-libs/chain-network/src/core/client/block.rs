use crate::data::BlockId;
use crate::error::Error;
use async_trait::async_trait;

/// Interface for the blockchain node service responsible for
/// providing access to blocks.
#[async_trait]
pub trait BlockService {
    async fn handshake(&mut self) -> Result<BlockId, HandshakeError>;
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
