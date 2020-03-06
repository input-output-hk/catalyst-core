use super::convert;
use super::proto;
use super::streaming::{InboundStream, OutboundStream};
use crate::data::{gossip::Peers, Block, BlockEvent, BlockId, BlockIds, Header};
use crate::error::{Error, HandshakeError};
use crate::PROTOCOL_VERSION;
use futures::prelude::*;
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

impl<T> Client<T>
where
    T: GrpcService<BoxBody> + Send,
    T::Future: Send,
    T::ResponseBody: Body + HttpBody + Send + 'static,
    T::Error: Into<StdError>,
    <T::ResponseBody as HttpBody>::Error: Into<StdError> + Send,
{
    /// Requests the identifier of the genesis block from the service node.
    ///
    /// The implementation can also perform version information checks to
    /// ascertain that the client use compatible protocol versions.
    ///
    /// This method should be called first after establishing the client
    /// connection.
    pub async fn handshake(&mut self) -> Result<BlockId, HandshakeError> {
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

    /// Requests the header of the tip block in the node's chain.
    pub async fn tip(&mut self) -> Result<Header, Error> {
        let req = proto::TipRequest {};
        let res = self.inner.tip(req).await?.into_inner();
        let header = Header::from_bytes(res.block_header);
        Ok(header)
    }

    /// Requests the identified blocks in a streamed response.
    pub async fn get_blocks(
        &mut self,
        ids: BlockIds,
    ) -> Result<InboundStream<proto::Block, Block>, Error> {
        let ids = proto::BlockIds {
            ids: convert::ids_into_repeated_bytes(ids),
        };
        let stream = self.inner.get_blocks(ids).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// Establishes a bidirectional stream of notifications for blocks
    /// created or accepted by either of the peers.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn block_subscription<S>(
        &mut self,
        outbound: S,
    ) -> Result<InboundStream<proto::BlockEvent, BlockEvent>, Error>
    where
        S: Stream<Item = Header> + Send + Sync + 'static,
    {
        let outbound = OutboundStream::new(outbound);
        let inbound = self.inner.block_subscription(outbound).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }

    /// One-off request for a list of peers known to the remote node.
    ///
    /// The peers are picked up accordingly to the Poldercast algorithm
    /// modules. This request is typically used during bootstrap from
    /// a trusted peer.
    pub async fn peers(&mut self, limit: u32) -> Result<Peers, Error> {
        let req = proto::PeersRequest { limit };
        let res = self.inner.peers(req).await?.into_inner();
        let peers = convert::from_protobuf_repeated(res.peers)?;
        Ok(peers)
    }
}
