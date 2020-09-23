use crate::error::{Code, Error};

use std::convert::TryFrom;
use std::fmt;
use std::net::SocketAddr;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Peer {
    addr: SocketAddr,
}

pub type Peers = Box<[Peer]>;

impl Peer {
    #[inline]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl From<SocketAddr> for Peer {
    #[inline]
    fn from(addr: SocketAddr) -> Self {
        Peer { addr }
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.addr)
    }
}

const NODE_ID_LEN: usize = 32;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId([u8; NODE_ID_LEN]);

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("NodeId(0x")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        f.write_str(")")
    }
}

impl NodeId {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn authenticated(self, signature: &[u8]) -> Result<AuthenticatedNodeId, Error> {
        if signature.len() != NODE_SIGNATURE_LEN {
            return Err(Error::new(
                Code::InvalidArgument,
                format!("node signature must be {} bytes long", NODE_SIGNATURE_LEN),
            ));
        }
        let mut res = AuthenticatedNodeId {
            id: self,
            signature: [0; NODE_SIGNATURE_LEN],
        };
        res.signature.copy_from_slice(&signature);
        Ok(res)
    }
}

impl TryFrom<&[u8]> for NodeId {
    type Error = Error;

    fn try_from(src: &[u8]) -> Result<Self, Error> {
        match TryFrom::try_from(src) {
            Ok(data) => Ok(NodeId(data)),
            Err(_) => Err(Error::new(
                Code::InvalidArgument,
                format!("block identifier must be {} bytes long", NODE_ID_LEN),
            )),
        }
    }
}

const NODE_SIGNATURE_LEN: usize = 64;

pub struct AuthenticatedNodeId {
    id: NodeId,
    signature: [u8; NODE_SIGNATURE_LEN],
}

impl AuthenticatedNodeId {
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl From<AuthenticatedNodeId> for NodeId {
    fn from(auth: AuthenticatedNodeId) -> Self {
        auth.id
    }
}
