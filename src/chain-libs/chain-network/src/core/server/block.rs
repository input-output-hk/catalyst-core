use super::push::ResponseHandler;
use crate::data::{Block, BlockEvent, BlockId, BlockIds, Header};
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

    /// Stream blocks from either of the given starting points
    /// to the server's tip.
    async fn pull_blocks_to_tip(
        &self,
        from: BlockIds,
    ) -> Result<Self::PullBlocksToTipStream, Error>;

    type PushHeadersSink: Sink<Header, Error = Error> + Send;
    type PushHeadersResponseHandler: ResponseHandler<Response = ()> + Send;

    /// Called by the protocol implementation to handle a stream
    /// of block headers sent by the peer in response to a
    /// `BlockEvent::Missing` solicitation.
    ///
    /// Returns a sink object and a push handler object.
    async fn push_headers(
        &self,
    ) -> Result<(Self::PushHeadersSink, Self::PushHeadersResponseHandler), Error>;

    type UploadBlocksSink: Sink<Block, Error = Error> + Send;
    type UploadBlocksResponseHandler: ResponseHandler<Response = ()> + Send;

    /// Called by the protocol implementation to handle a stream
    /// of blocks sent by the peer in response to a
    /// `BlockEvent::Solicit` solicitation.
    ///
    /// Returns a sink object and a push handler object.
    async fn upload_blocks(
        &self,
    ) -> Result<(Self::UploadBlocksSink, Self::UploadBlocksResponseHandler), Error>;

    type SubscriptionStream: Stream<Item = Result<BlockEvent, Error>> + Send + Sync;
    type SubscriptionSink: Sink<Header, Error = Error> + Send + Sync;
    type SubscriptionResponseHandler: ResponseHandler<Response = ()> + Send + Sync;

    /// Provides objects to manage a bidirectional subscription
    /// for announcing blocks.
    ///
    /// The implementation of the method returns a future resolving
    /// to the triplet of:
    ///
    /// - an outbound asynchronous stream used by this node to send
    ///   block announcements and solicitations,
    /// - an asynchronous sink for inbound block announcements,
    /// - a push handler to process completion of the inbound stream.
    async fn subscription(
        &self,
    ) -> Result<
        (
            Self::SubscriptionStream,
            Self::SubscriptionSink,
            Self::SubscriptionResponseHandler,
        ),
        Error,
    >;
}
