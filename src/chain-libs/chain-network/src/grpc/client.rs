use super::convert;
use super::proto;
use super::streaming::{InboundStream, OutboundStream};
use crate::data::block::{Block, BlockEvent, BlockId, BlockIds, Header};
use crate::data::fragment::{Fragment, FragmentIds};
use crate::data::{Gossip, Peer, Peers};
use crate::error::{Error, HandshakeError};
use crate::PROTOCOL_VERSION;
use futures::prelude::*;
use tonic::body::{Body, BoxBody};
use tonic::client::GrpcService;
use tonic::codegen::{HttpBody, StdError};
use tonic::Request;

use std::convert::TryFrom;

#[derive(Clone)]
pub struct Client<T> {
    inner: proto::node_client::NodeClient<T>,
    public_peer: Option<Peer>,
}

/// The inbound subscription stream of block events.
pub type BlockSubscription = InboundStream<proto::BlockEvent, BlockEvent>;

/// The inbound subscription stream of fragments.
pub type FragmentSubscription = InboundStream<proto::Fragment, Fragment>;

/// The inbound subscription stream of P2P gossip.
pub type GossipSubscription = InboundStream<proto::Gossip, Gossip>;

#[cfg(feature = "transport")]
impl Client<tonic::transport::Channel> {
    pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
    where
        D: std::convert::TryInto<tonic::transport::Endpoint>,
        D::Error: Into<StdError>,
    {
        let inner = proto::node_client::NodeClient::connect(dst).await?;
        Ok(Client {
            inner,
            public_peer: None,
        })
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
            public_peer: None,
        }
    }

    /// Sets public peer address advertized to the remote peer
    /// in subscription requests.
    pub fn set_public_peer(&mut self, peer: Peer) {
        self.public_peer = Some(peer);
    }

    fn new_subscription_request<S>(&self, outbound: S) -> Request<OutboundStream<S>> {
        let mut req = Request::new(OutboundStream::new(outbound));
        if let Some(peer) = &self.public_peer {
            convert::encode_peer(peer, req.metadata_mut());
        }
        req
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
            ids: convert::ids_into_repeated_bytes(ids.iter()),
        };
        let stream = self.inner.get_blocks(ids).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// Requests the headers of the identified blocks in a streamed response.
    pub async fn get_headers(
        &mut self,
        ids: BlockIds,
    ) -> Result<InboundStream<proto::Header, Header>, Error> {
        let ids = proto::BlockIds {
            ids: convert::ids_into_repeated_bytes(ids.iter()),
        };
        let stream = self.inner.get_headers(ids).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// Requests the identified fragments in a streamed response.
    pub async fn get_fragments(
        &mut self,
        ids: FragmentIds,
    ) -> Result<InboundStream<proto::Fragment, Fragment>, Error> {
        let ids = proto::FragmentIds {
            ids: convert::ids_into_repeated_bytes(ids.into_vec()),
        };
        let stream = self.inner.get_fragments(ids).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// Stream blocks from the first of the given starting points
    /// that is found in the peer's chain, to the chain's tip.
    pub async fn pull_blocks_to_tip(
        &mut self,
        from: BlockIds,
    ) -> Result<InboundStream<proto::Block, Block>, Error> {
        let req = proto::PullBlocksToTipRequest {
            from: convert::ids_into_repeated_bytes(from.into_vec()),
        };
        let stream = self.inner.pull_blocks_to_tip(req).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// Requests headers of blocks in the blockchain's chronological order,
    /// in the range between the latest of the given starting points, and
    /// the given ending point. If none of the starting points are found
    /// in the chain on the service side, or if the ending point is not found,
    /// the future will fail with a `NotFound` error.
    pub async fn pull_headers(
        &mut self,
        from: BlockIds,
        to: BlockId,
    ) -> Result<InboundStream<proto::Header, Header>, Error> {
        let req = proto::PullHeadersRequest {
            from: convert::ids_into_repeated_bytes(from.into_vec()),
            to: to.as_bytes().into(),
        };
        let stream = self.inner.pull_headers(req).await?.into_inner();
        Ok(InboundStream::new(stream))
    }

    /// The outbound counterpart of `pull_headers`, called in response to a
    /// `BlockEvent::Missing` solicitation.
    /// An empty stream can be used to indicate that the solicitation
    /// does not refer to blocks found in the local blockchain.
    pub async fn push_headers<S>(&mut self, headers: S) -> Result<(), Error>
    where
        S: Stream<Item = Header> + Send + Sync + 'static,
    {
        let outbound = OutboundStream::new(headers);
        let proto::PushHeadersResponse {} = self.inner.push_headers(outbound).await?.into_inner();
        Ok(())
    }

    /// Uploads blocks to the service in response to `BlockEvent::Solicit`.
    ///
    /// The blocks to send are retrieved asynchronously from the passed stream.
    pub async fn upload_blocks<S>(&mut self, blocks: S) -> Result<(), Error>
    where
        S: Stream<Item = Block> + Send + Sync + 'static,
    {
        let outbound = OutboundStream::new(blocks);
        let proto::UploadBlocksResponse {} = self.inner.upload_blocks(outbound).await?.into_inner();
        Ok(())
    }

    /// Establishes a bidirectional stream of notifications for blocks
    /// created or accepted by either of the peers.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn block_subscription<S>(&mut self, outbound: S) -> Result<BlockSubscription, Error>
    where
        S: Stream<Item = Header> + Send + Sync + 'static,
    {
        let req = self.new_subscription_request(outbound);
        let inbound = self.inner.block_subscription(req).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }

    /// Establishes a bidirectional stream for exchanging fragments
    /// created or accepted by either of the peers.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn fragment_subscription<S>(
        &mut self,
        outbound: S,
    ) -> Result<FragmentSubscription, Error>
    where
        S: Stream<Item = Fragment> + Send + Sync + 'static,
    {
        let req = self.new_subscription_request(outbound);
        let inbound = self.inner.fragment_subscription(req).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }

    /// Establishes a bidirectional stream for exchanging network gossip.
    ///
    /// The client can use the stream that the returned future resolves to
    /// as a long-lived subscription handle.
    pub async fn gossip_subscription<S>(&mut self, outbound: S) -> Result<GossipSubscription, Error>
    where
        S: Stream<Item = Gossip> + Send + Sync + 'static,
    {
        let req = self.new_subscription_request(outbound);
        let inbound = self.inner.gossip_subscription(req).await?.into_inner();
        Ok(InboundStream::new(inbound))
    }
}
