use crate::error::{Code, Error};
use chain_crypto::{Ed25519, KeyPair, PublicKey, Signature, Verification};
use rand_core::{CryptoRng, RngCore};

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

/// The key pair used to authenticate a network node,
/// including the secret key.
pub struct NodeKeyPair(KeyPair<Ed25519>);

impl NodeKeyPair {
    /// Generates a key pair using the provided random number generator.
    pub fn generate<R: RngCore + CryptoRng>(rng: R) -> Self {
        NodeKeyPair(KeyPair::generate(rng))
    }

    /// Produces the node ID (i.e. the public key), authenticated by signing
    /// the provided nonce with the secret key.
    pub fn sign(&self, nonce: &[u8]) -> AuthenticatedNodeId {
        let signature = self.0.private_key().sign(nonce);
        AuthenticatedNodeId {
            id: NodeId(self.0.public_key().clone()),
            signature,
        }
    }
}

/// Identifier of a network peer.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(PublicKey<Ed25519>);

impl NodeId {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0.as_ref()
    }

    /// Adds a signature given as a byte slice to produce an
    /// `AuthenticatedNodeId`.
    ///
    /// # Errors
    ///
    /// Returns an error if `signature` byte slice does not conform to the
    /// format of a signature used to authenticate node ID.
    pub fn authenticated(self, signature: &[u8]) -> Result<AuthenticatedNodeId, Error> {
        let signature =
            Signature::from_binary(signature).map_err(|e| Error::new(Code::InvalidArgument, e))?;
        Ok(AuthenticatedNodeId {
            id: self,
            signature,
        })
    }
}

impl TryFrom<&[u8]> for NodeId {
    type Error = Error;

    fn try_from(src: &[u8]) -> Result<Self, Error> {
        match PublicKey::from_binary(src) {
            Ok(data) => Ok(NodeId(data)),
            Err(e) => Err(Error::new(Code::InvalidArgument, e)),
        }
    }
}

/// A node ID accompanied with a signature.
///
/// The signature is not assumed to be valid by construction.
/// Use the `verify` method to verify the signature against the original
/// nonce.
pub struct AuthenticatedNodeId {
    id: NodeId,
    signature: Signature<[u8], Ed25519>,
}

impl AuthenticatedNodeId {
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature.as_ref()
    }

    /// Verifies that the signature is correct for this node ID and
    /// the given nonce.
    pub fn verify(&self, nonce: &[u8]) -> Result<(), Error> {
        match self.signature.verify(&self.id.0, nonce) {
            Verification::Success => Ok(()),
            Verification::Failed => Err(Error::new(
                Code::InvalidArgument,
                "invalid node ID signature",
            )),
        }
    }
}

impl From<AuthenticatedNodeId> for NodeId {
    fn from(auth: AuthenticatedNodeId) -> Self {
        auth.id
    }
}
