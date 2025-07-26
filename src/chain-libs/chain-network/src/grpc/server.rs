use super::proto;
use super::streaming::{InboundStream, OutboundTryStream};

#[cfg(feature = "legacy")]
use super::legacy;

use crate::core::server::{BlockService, FragmentService, GossipService, Node};
use crate::data::p2p::NodeId;
use crate::data::{block, fragment, BlockId, Peer};
use crate::PROTOCOL_VERSION;
use tonic::{Code, Status};

#[cfg(feature = "legacy")]
use tonic::metadata::MetadataValue;

use std::net::SocketAddr;

pub type Server<T> = proto::node::node_server::NodeServer<NodeService<T>>;

/// Builder to customize the gRPC server.
#[derive(Default)]
pub struct Builder {
    #[cfg(feature = "legacy")]
    legacy_node_id: Option<legacy::NodeId>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            #[cfg(feature = "legacy")]
            legacy_node_id: None,
        }
    }

    /// Make the server add "node-id-bin" metadata with the passed value
    /// into subscription responses, for backward compatibility with
    /// jormungandr versions prior to 0.9.
    #[cfg(feature = "legacy")]
    pub fn legacy_node_id(&mut self, node_id: legacy::NodeId) -> &mut Self {
        self.legacy_node_id = Some(node_id);
        self
    }

    pub fn build<T: Node>(&self, inner: T) -> Server<T> {
        let service = NodeService {
            #[cfg(feature = "legacy")]
            legacy_node_id: self.legacy_node_id,
            ..NodeService::new(inner)
        };
        Server::new(service)
    }
}

#[derive(Debug)]
pub struct NodeService<T> {
    inner: T,
    #[cfg(feature = "legacy")]
    legacy_node_id: Option<legacy::NodeId>,
}

impl<T> NodeService<T>
where
    T: Node,
{
    pub fn new(inner: T) -> Self {
        NodeService {
            inner,
            #[cfg(feature = "legacy")]
            legacy_node_id: None,
        }
    }

    fn block_service(&self) -> Result<&T::BlockService, Box<Status>> {
        self.inner
            .block_service()
            .ok_or_else(|| Box::new(Status::new(Code::Unimplemented, "not implemented")))
    }

    fn fragment_service(&self) -> Result<&T::FragmentService, Box<Status>> {
        self.inner
            .fragment_service()
            .ok_or_else(|| Box::new(Status::new(Code::Unimplemented, "not implemented")))
    }

    fn gossip_service(&self) -> Result<&T::GossipService, Box<Status>> {
        self.inner
            .gossip_service()
            .ok_or_else(|| Box::new(Status::new(Code::Unimplemented, "not implemented")))
    }

    #[allow(unused_mut)]
    #[allow(clippy::let_and_return)]
    fn subscription_response<S>(&self, outbound: S) -> tonic::Response<OutboundTryStream<S>> {
        let mut res = tonic::Response::new(OutboundTryStream::new(outbound));
        #[cfg(feature = "legacy")]
        if let Some(node_id) = self.legacy_node_id {
            let val = MetadataValue::from_bytes(&node_id.encode());
            res.metadata_mut().insert_bin("node-id-bin", val);
        }
        res
    }
}

fn remote_addr_to_peer(maybe_addr: Option<SocketAddr>) -> Result<Peer, Box<Status>> {
    match maybe_addr {
        Some(addr) => Ok(addr.into()),
        None => Err(Box::new(Status::internal(
            "transport does not provide the remote address",
        ))),
    }
}

#[tonic::async_trait]
impl<T> proto::node::node_server::Node for NodeService<T>
where
    T: Node,
{
    async fn handshake(
        &self,
        req: tonic::Request<proto::node::HandshakeRequest>,
    ) -> Result<tonic::Response<proto::node::HandshakeResponse>, tonic::Status> {
        let peer = remote_addr_to_peer(req.remote_addr()).map_err(|e| *e)?;
        let req = req.into_inner();
        let nonce = &req.nonce;
        let hr = self.inner.handshake(peer, nonce).await?;
        let res = proto::node::HandshakeResponse {
            version: PROTOCOL_VERSION,
            block0: hr.block0_id.as_bytes().into(),
            node_id: hr.auth.id().as_bytes().into(),
            signature: hr.auth.signature().into(),
            nonce: hr.nonce.into(),
        };
        Ok(tonic::Response::new(res))
    }

    async fn client_auth(
        &self,
        req: tonic::Request<proto::node::ClientAuthRequest>,
    ) -> Result<tonic::Response<proto::node::ClientAuthResponse>, tonic::Status> {
        let peer = remote_addr_to_peer(req.remote_addr()).map_err(|e| *e)?;
        let req = req.into_inner();
        let node_id = NodeId::try_from(&req.node_id[..])?;
        let auth = node_id.authenticated(&req.signature)?;
        self.inner.client_auth(peer, auth).await?;
        let res = proto::node::ClientAuthResponse {};
        Ok(tonic::Response::new(res))
    }

    async fn tip(
        &self,
        _: tonic::Request<proto::node::TipRequest>,
    ) -> Result<tonic::Response<proto::node::TipResponse>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let header = service.tip().await?;
        let res = proto::node::TipResponse {
            block_header: header.into(),
        };
        Ok(tonic::Response::new(res))
    }

    async fn peers(
        &self,
        req: tonic::Request<proto::node::PeersRequest>,
    ) -> Result<tonic::Response<proto::node::PeersResponse>, tonic::Status> {
        let service = self.gossip_service().map_err(|e| *e)?;
        let peers = service.peers(req.into_inner().limit).await?;
        let res = proto::node::PeersResponse {
            peers: peers
                .nodes
                .iter()
                .cloned()
                .map(|node| node.into_bytes())
                .collect(),
        };
        Ok(tonic::Response::new(res))
    }

    type GetBlocksStream = OutboundTryStream<<T::BlockService as BlockService>::GetBlocksStream>;

    async fn get_blocks(
        &self,
        req: tonic::Request<proto::types::BlockIds>,
    ) -> Result<tonic::Response<Self::GetBlocksStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let ids = block::try_ids_from_iter(req.into_inner().ids)?;
        let stream = service.get_blocks(ids).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type GetHeadersStream = OutboundTryStream<<T::BlockService as BlockService>::GetHeadersStream>;

    async fn get_headers(
        &self,
        req: tonic::Request<proto::types::BlockIds>,
    ) -> Result<tonic::Response<Self::GetHeadersStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let ids = block::try_ids_from_iter(req.into_inner().ids)?;
        let stream = service.get_headers(ids).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type GetFragmentsStream =
        OutboundTryStream<<T::FragmentService as FragmentService>::GetFragmentsStream>;

    async fn get_fragments(
        &self,
        req: tonic::Request<proto::types::FragmentIds>,
    ) -> Result<tonic::Response<Self::GetFragmentsStream>, tonic::Status> {
        let service = self.fragment_service().map_err(|e| *e)?;
        let ids = fragment::try_ids_from_iter(req.into_inner().ids)?;
        let stream = service.get_fragments(ids).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type PullHeadersStream =
        OutboundTryStream<<T::BlockService as BlockService>::PullHeadersStream>;

    async fn pull_headers(
        &self,
        req: tonic::Request<proto::node::PullHeadersRequest>,
    ) -> Result<tonic::Response<Self::PullHeadersStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let (from, to) = {
            let req = req.into_inner();
            (
                block::try_ids_from_iter(req.from)?,
                BlockId::try_from(&req.to[..])?,
            )
        };
        let stream = service.pull_headers(from, to).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type PullBlocksStream = OutboundTryStream<<T::BlockService as BlockService>::PullBlocksStream>;

    async fn pull_blocks(
        &self,
        req: tonic::Request<proto::node::PullBlocksRequest>,
    ) -> Result<tonic::Response<Self::PullBlocksStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let req = req.into_inner();
        let from = block::try_ids_from_iter(req.from)?;
        let to = BlockId::try_from(&req.to[..])?;
        let stream = service.pull_blocks(from, to).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    type PullBlocksToTipStream =
        OutboundTryStream<<T::BlockService as BlockService>::PullBlocksToTipStream>;

    async fn pull_blocks_to_tip(
        &self,
        req: tonic::Request<proto::node::PullBlocksToTipRequest>,
    ) -> Result<tonic::Response<Self::PullBlocksToTipStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let from = block::try_ids_from_iter(req.into_inner().from)?;
        let stream = service.pull_blocks_to_tip(from).await?;
        Ok(tonic::Response::new(OutboundTryStream::new(stream)))
    }

    async fn push_headers(
        &self,
        req: tonic::Request<tonic::Streaming<proto::types::Header>>,
    ) -> Result<tonic::Response<proto::node::PushHeadersResponse>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let stream = InboundStream::new(req.into_inner());
        service.push_headers(Box::pin(stream)).await?;
        Ok(tonic::Response::new(proto::node::PushHeadersResponse {}))
    }

    async fn upload_blocks(
        &self,
        req: tonic::Request<tonic::Streaming<proto::types::Block>>,
    ) -> Result<tonic::Response<proto::node::UploadBlocksResponse>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let stream = InboundStream::new(req.into_inner());
        service.upload_blocks(Box::pin(stream)).await?;
        Ok(tonic::Response::new(proto::node::UploadBlocksResponse {}))
    }

    type BlockSubscriptionStream =
        OutboundTryStream<<T::BlockService as BlockService>::SubscriptionStream>;

    async fn block_subscription(
        &self,
        req: tonic::Request<tonic::Streaming<proto::types::Header>>,
    ) -> Result<tonic::Response<Self::BlockSubscriptionStream>, tonic::Status> {
        let service = self.block_service().map_err(|e| *e)?;
        let peer = remote_addr_to_peer(req.remote_addr()).map_err(|e| *e)?;
        let inbound = InboundStream::new(req.into_inner());
        let outbound = service.block_subscription(peer, Box::pin(inbound)).await?;
        let res = self.subscription_response(outbound);
        Ok(res)
    }

    type FragmentSubscriptionStream =
        OutboundTryStream<<T::FragmentService as FragmentService>::SubscriptionStream>;

    async fn fragment_subscription(
        &self,
        req: tonic::Request<tonic::Streaming<proto::types::Fragment>>,
    ) -> Result<tonic::Response<Self::FragmentSubscriptionStream>, tonic::Status> {
        let service = self.fragment_service().map_err(|e| *e)?;
        let peer = remote_addr_to_peer(req.remote_addr()).map_err(|e| *e)?;
        let inbound = InboundStream::new(req.into_inner());
        let outbound = service
            .fragment_subscription(peer, Box::pin(inbound))
            .await?;
        let res = self.subscription_response(outbound);
        Ok(res)
    }

    type GossipSubscriptionStream =
        OutboundTryStream<<T::GossipService as GossipService>::SubscriptionStream>;

    async fn gossip_subscription(
        &self,
        req: tonic::Request<tonic::Streaming<proto::node::Gossip>>,
    ) -> Result<tonic::Response<Self::GossipSubscriptionStream>, tonic::Status> {
        let service = self.gossip_service().map_err(|e| *e)?;
        let peer = remote_addr_to_peer(req.remote_addr()).map_err(|e| *e)?;
        let inbound = InboundStream::new(req.into_inner());
        let outbound = service.gossip_subscription(peer, Box::pin(inbound)).await?;
        let res = self.subscription_response(outbound);
        Ok(res)
    }
}
