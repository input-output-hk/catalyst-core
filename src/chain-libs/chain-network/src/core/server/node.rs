use super::{BlockService, FragmentService, GossipService};
use crate::data::p2p::{AuthenticatedNodeId, Peer};
use crate::data::HandshakeResponse;
use crate::error::Error;
use async_trait::async_trait;

/// Interface to application logic of the blockchain node server.
///
/// An implementation of a blockchain node implements this trait to
/// serve the network protocols using node's subsystems such as
/// block storage and fragment processor.
#[async_trait]
pub trait Node: Send + Sync + 'static {
    /// The implementation of the block service.
    type BlockService: BlockService + Send + Sync;

    /// The implementation of the fragment service.
    type FragmentService: FragmentService + Send + Sync;

    /// The implementation of the gossip service.
    type GossipService: GossipService + Send + Sync;

    /// Implements node handshake. The server returns the ID of the genesis
    /// block and its own node ID, authenticated with the signature of `nonce`.
    async fn handshake(&self, peer: Peer, nonce: &[u8]) -> Result<HandshakeResponse, Error>;

    /// Handles client ID authentication.
    async fn client_auth(&self, peer: Peer, auth: AuthenticatedNodeId) -> Result<(), Error>;

    /// Instantiates the block service,
    /// if supported by this node.
    fn block_service(&self) -> Option<&Self::BlockService>;

    /// Instantiates the fragment service,
    /// if supported by this node.
    fn fragment_service(&self) -> Option<&Self::FragmentService>;

    /// Instantiates the gossip service,
    /// if supported by this node.
    fn gossip_service(&self) -> Option<&Self::GossipService>;
}
