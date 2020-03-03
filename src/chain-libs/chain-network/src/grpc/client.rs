use super::convert;
use super::proto;
use super::streaming::InboundStream;
use crate::core::client::{BlockService, GossipService, HandshakeError};
use crate::data::{block, gossip::Peers, Block, BlockId, BlockIds, Header};
use crate::error::Error;
use crate::PROTOCOL_VERSION;
use async_trait::async_trait;
use tonic::body::{Body, BoxBody};
use tonic::client::GrpcService;
use tonic::codegen::{HttpBody, StdError};

use std::convert::TryFrom;

#[derive(Clone)]
pub struct Client<T> {
    inner: proto::node_client::NodeClient<T>,
}

#[cfg(feature = "transport")]
impl Client<tonic::transport::Channel> {
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let inner = proto::node_client::NodeClient::connect(dst).await?;
        Ok(Client { inner })
    }
}

impl<T> Client<T>
where
    T: GrpcService<BoxBody>,
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
{
    pub fn new(service: T) -> Self {
        Client {
            inner: proto::node_client::NodeClient::new(service),
        }
    }
}

#[async_trait]
impl<T> BlockService for Client<T>
where
    T: GrpcService<BoxBody> + Send,
    T::Future: Send,
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
{
    async fn handshake(&mut self) -> Result<BlockId, HandshakeError> {
        let req = proto::HandshakeRequest {};
        let res = self
            .inner
            .handshake(req)
            .await
            .map_err(|status| HandshakeError::Rpc(convert::error_from_grpc(status)))?
            .into_inner();
        if res.version != PROTOCOL_VERSION {
            return Err(HandshakeError::UnsupportedVersion(
                res.version.to_string().into(),
            ));
        }
        BlockId::try_from(&res.block0[..]).map_err(HandshakeError::InvalidBlock0)
    }

    async fn tip(&mut self) -> Result<Header, Error> {
        let req = proto::TipRequest {};
        let res = self.inner.tip(req).await?.into_inner();
        let header = Header::from_bytes(res.block_header);
        Ok(header)
    }

    type GetBlocksStream = InboundStream<proto::Block, Block>;

    async fn get_blocks(&mut self, ids: BlockIds) -> Result<Self::GetBlocksStream, Error> {
        let ids = proto::BlockIds {
            ids: block::ids_into_repeated_bytes(ids),
        };
        let stream = self.inner.get_blocks(ids).await?.into_inner();
        Ok(InboundStream::new(stream))
    }
}

#[async_trait]
impl<T> GossipService for Client<T>
where
    T: GrpcService<BoxBody> + Send,
    T::Future: Send,
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
{
    async fn peers(&mut self, limit: u32) -> Result<Peers, Error> {
        let req = proto::PeersRequest { limit };
        let res = self.inner.peers(req).await?.into_inner();
        let peers = convert::from_protobuf_repeated(res.peers)?;
        Ok(peers)
    }
}
