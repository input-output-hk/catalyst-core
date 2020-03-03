use crate::data::gossip::Peers;
use crate::error::Error;
use async_trait::async_trait;

/// Client-side interface for the remote blockchain node service
/// responsible for exchanging P2P network gossip.
#[async_trait]
pub trait GossipService {
    /// One-off request for a list of peers known to the remote node.
    ///
    /// The peers are picked up accordingly to the Poldercast algorithm
    /// modules. This request is typically used during bootstrap from
    /// a trusted peer.
    async fn peers(&mut self, limit: u32) -> Result<Peers, Error>;
}
