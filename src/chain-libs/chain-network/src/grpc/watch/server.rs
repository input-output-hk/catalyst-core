use crate::core::watch::server::Watch;
use crate::grpc::proto;
use crate::grpc::streaming::OutboundTryStream;

#[derive(Debug)]
pub struct WatchService<T> {
    inner: T,
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
        todo!()
    }

    type TipSubscriptionStream = OutboundTryStream<T::TipSubscriptionStream>;

    async fn tip_subscription(
        &self,
        request: tonic::Request<proto::watch::TipSubscriptionRequest>,
    ) -> Result<tonic::Response<Self::TipSubscriptionStream>, tonic::Status> {
        todo!()
    }

    type SyncMultiverseStream = OutboundTryStream<T::SyncMultiverseStream>;

    async fn sync_multiverse(
        &self,
        request: tonic::Request<proto::watch::SyncMultiverseRequest>,
    ) -> Result<tonic::Response<Self::SyncMultiverseStream>, tonic::Status> {
        let proto::watch::SyncMultiverseRequest { from } = request.into_inner();
        todo!()
    }
}
