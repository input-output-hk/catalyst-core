use crate::data::block::{Block, BlockId, Header};
use crate::error::Error;
use crate::grpc::convert;
use crate::grpc::proto;
use crate::grpc::streaming::InboundStream;

use http_body::Body;
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::StdError;

#[cfg(feature = "transport")]
use tonic::transport;

pub struct Client<T> {
    inner: proto::watch::watch_client::WatchClient<T>,
}

#[cfg(feature = "transport")]
impl Client<transport::Channel> {
    pub async fn connect<D>(dst: D) -> Result<Self, transport::Error>
    where
        D: TryInto<transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let inner = proto::watch::watch_client::WatchClient::connect(dst).await?;
        Ok(Client { inner })
    }
}

/// The inbound subscription stream of block events.
pub type BlockSubscription = InboundStream<proto::types::Block, Block>;

/// The inbound subscription stream of blockchain tip headers.
pub type TipSubscription = InboundStream<proto::types::Header, Header>;

/// The inbound stream of blocks sent in response to a
/// [`Client::sync_multiverse`] request.
pub type SyncMultiverseStream = InboundStream<proto::types::Block, Block>;

impl<T> Client<T>
where
    T: GrpcService<BoxBody>,
    T::ResponseBody: Send + Sync + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Establishes a subscription for blocks that have been issued by the node
    /// or received from the p2p network.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn block_subscription(&mut self) -> Result<BlockSubscription, Error> {
        let req = tonic::Request::new(proto::watch::BlockSubscriptionRequest {});
        let inbound = self.inner.block_subscription(req).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }

    /// Establishes a subscription for tip change announcements by the node.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn tip_subscription(&mut self) -> Result<TipSubscription, Error> {
        let req = tonic::Request::new(proto::watch::TipSubscriptionRequest {});
        let inbound = self.inner.tip_subscription(req).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }

    pub async fn sync_multiverse(
        &mut self,
        from: impl IntoIterator<Item = &BlockId>,
    ) -> Result<SyncMultiverseStream, Error> {
        let req = proto::watch::SyncMultiverseRequest {
            from: convert::ids_into_repeated_bytes(from),
        };
        let stream = self.inner.sync_multiverse(req).await?.into_inner();
        Ok(InboundStream::new(stream))
    }
}
