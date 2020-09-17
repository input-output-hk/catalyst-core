use super::block::BlockId;
use super::p2p::AuthenticatedNodeId;

pub struct HandshakeResponse {
    pub block0_id: BlockId,
    pub auth: AuthenticatedNodeId,
    pub nonce: Box<[u8]>,
}
