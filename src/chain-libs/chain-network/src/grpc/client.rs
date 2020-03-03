use super::convert::error_from_grpc;
use super::proto;
use crate::core::client::{BlockService, HandshakeError};
use crate::data::BlockId;
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
            .map_err(|status| HandshakeError::Rpc(error_from_grpc(status)))?
            .into_inner();
        if res.version != PROTOCOL_VERSION {
            return Err(HandshakeError::UnsupportedVersion(
                res.version.to_string().into(),
            ));
        }
        BlockId::try_from(&res.block0[..]).map_err(HandshakeError::InvalidBlock0)
    }
}
