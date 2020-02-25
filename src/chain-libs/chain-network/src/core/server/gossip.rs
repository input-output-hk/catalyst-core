use crate::data::gossip::Peers;
use crate::error::Error;
use async_trait::async_trait;

/// Interface for the blockchain node service implementation responsible for
/// exchanging P2P network gossip.
#[async_trait]
pub trait GossipService {
    async fn peers(&self, limit: u32) -> Result<Peers, Error>;
}
