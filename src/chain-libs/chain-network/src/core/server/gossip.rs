use super::PushStream;
use crate::data::{Gossip, Peer, Peers};
use crate::error::Error;
use async_trait::async_trait;
use futures::stream::Stream;

/// Interface for the blockchain node service implementation responsible for
/// exchanging P2P network gossip.
#[async_trait]
pub trait GossipService {
    async fn peers(&self, limit: u32) -> Result<Peers, Error>;

    /// The type of outbound asynchronous streams returned by the
    /// `subscription` method.
    type SubscriptionStream: Stream<Item = Result<Gossip, Error>> + Send + Sync;

    /// Called by the protocol implementation to establish a
    /// bidirectional subscription stream.
    /// The inbound stream is passed to the asynchronous method,
    /// which resolves to the outbound stream.
    async fn gossip_subscription(
        &self,
        subscriber: Peer,
        stream: PushStream<Gossip>,
    ) -> Result<Self::SubscriptionStream, Error>;
}
