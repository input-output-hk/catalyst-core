use crate::core::watch::server::Watch;
use crate::data::block;
use crate::grpc::proto;
use crate::grpc::streaming::OutboundTryStream;

pub type Server<T> = proto::watch::watch_server::WatchServer<WatchService<T>>;

#[derive(Debug)]
pub struct WatchService<T> {
    inner: T,
}

impl<T> WatchService<T> {
    pub fn new(inner: T) -> Self {
        WatchService { inner }
    }
}

#[tonic::async_trait]
impl<T> proto::watch::watch_server::Watch for WatchService<T>
where
    T: Watch,
{
    type BlockSubscriptionStream = OutboundTryStream<T::BlockSubscriptionStream>;

    async fn block_subscription(
        &self,
        request: tonic::Request<proto::watch::BlockSubscriptionRequest>,
    ) -> Result<tonic::Response<Self::BlockSubscriptionStream>, tonic::Status> {
        let proto::watch::BlockSubscriptionRequest {} = request.into_inner();
        let stream = self.inner.block_subscription().await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type TipSubscriptionStream = OutboundTryStream<T::TipSubscriptionStream>;

    async fn tip_subscription(
        &self,
        request: tonic::Request<proto::watch::TipSubscriptionRequest>,
    ) -> Result<tonic::Response<Self::TipSubscriptionStream>, tonic::Status> {
        let proto::watch::TipSubscriptionRequest {} = request.into_inner();
        let stream = self.inner.tip_subscription().await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type SyncMultiverseStream = OutboundTryStream<T::SyncMultiverseStream>;

    async fn sync_multiverse(
        &self,
        request: tonic::Request<proto::watch::SyncMultiverseRequest>,
    ) -> Result<tonic::Response<Self::SyncMultiverseStream>, tonic::Status> {
        let proto::watch::SyncMultiverseRequest { from } = request.into_inner();
        let from = block::try_ids_from_iter(from)?;
        let stream = self.inner.sync_multiverse(from).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }
}
