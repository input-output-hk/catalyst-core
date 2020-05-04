use super::PushStream;
use crate::data::{Block, BlockEvent, BlockId, BlockIds, Header, Peer};
use crate::error::Error;
use async_trait::async_trait;
use futures::prelude::*;

/// Interface for the blockchain node service implementation responsible for
/// providing access to block data.
#[async_trait]
pub trait BlockService {
    /// Returns the ID of the genesis block of the chain served by this node.
    fn block0(&self) -> BlockId;

    /// Serves a request for the current blockchain tip.
    /// Resolves to the tip of the blockchain
    /// accepted by this node.
    async fn tip(&self) -> Result<Header, Error>;

    /// The type of an asynchronous stream that provides blocks in
    /// response to `get_blocks` method.
    type GetBlocksStream: Stream<Item = Result<Block, Error>> + Send + Sync;

    /// Serves a request to retrieve blocks identified by the list of `ids`
    /// Resloves to a stream of blocks to send to the remote client peer.
    async fn get_blocks(&self, ids: BlockIds) -> Result<Self::GetBlocksStream, Error>;

    /// The type of an asynchronous stream that provides headers in
    /// response to `get_headers` method.
    type GetHeadersStream: Stream<Item = Result<Header, Error>> + Send + Sync;

    /// Serves a request to retrieve block headers identified by the list of `ids`
    /// Resloves to a stream of headers to send to the remote client peer.
    async fn get_headers(&self, ids: BlockIds) -> Result<Self::GetHeadersStream, Error>;

    /// The type of an asynchronous stream that provides headers in
    /// response to `pull_headers` method.
    type PullHeadersStream: Stream<Item = Result<Header, Error>> + Send + Sync;

    /// Get blocks, walking forward in a range between either of the given
    /// starting points, and the ending point.
    async fn pull_headers(
        &self,
        from: BlockIds,
        to: BlockId,
    ) -> Result<Self::PullHeadersStream, Error>;

    /// The type of an asynchronous stream that provides blocks in
    /// response to `pull_blocks_to_tip` method.
    type PullBlocksToTipStream: Stream<Item = Result<Block, Error>> + Send + Sync;

    /// Stream blocks from the first of the given starting points that is
    /// found in the node's chain, to the chain's tip.
    async fn pull_blocks_to_tip(
        &self,
        from: BlockIds,
    ) -> Result<Self::PullBlocksToTipStream, Error>;

    /// Called by the protocol implementation to handle a stream
    /// of block headers sent by the peer in response to a
    /// `BlockEvent::Missing` solicitation.
    async fn push_headers(&self, stream: PushStream<Header>) -> Result<(), Error>;

    /// Called by the protocol implementation to handle a stream
    /// of blocks sent by the peer in response to a
    /// `BlockEvent::Solicit` solicitation.
    async fn upload_blocks(&self, stream: PushStream<Block>) -> Result<(), Error>;

    /// The type of outbound asynchronous streams returned by the
    /// `subscription` method.
    type SubscriptionStream: Stream<Item = Result<BlockEvent, Error>> + Send + Sync;

    /// Called by the protocol implementation to establish a
    /// bidirectional subscription stream.
    /// The inbound stream is passed to the asynchronous method,
    /// which resolves to the outbound stream.
    async fn block_subscription(
        &self,
        subscriber: Peer,
        stream: PushStream<Header>,
    ) -> Result<Self::SubscriptionStream, Error>;
}
