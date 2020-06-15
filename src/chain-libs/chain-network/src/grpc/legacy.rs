//! Support for legacy features.

use rand_core::RngCore;
use std::array::TryFromSliceError;
use std::convert::{TryFrom, TryInto};
use std::fmt;

const NODE_ID_LEN: usize = 24;

/// Represents a randomly generated node ID such as was present in subscription
/// requests and responses in Jormungandr versions prior to 0.9
/// (as implemented in the old `network-grpc` crate).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId([u8; NODE_ID_LEN]);

impl NodeId {
    pub fn generate(rng: &mut impl RngCore) -> Result<Self, rand_core::Error> {
        let mut bytes = [0; NODE_ID_LEN];
        rng.try_fill_bytes(&mut bytes)?;
        Ok(NodeId(bytes))
    }

    /// Get the node ID as a byte slice.
    ///
    /// This is mostly useful for display purposes such as logging.
    /// To get the wire format representation, use `encode`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Get the wire format representation that was used (sigh) by
    /// jormungandr releases prior to 0.9.
    pub fn encode(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(NODE_ID_LEN + 8);
        vec.extend_from_slice(&(NODE_ID_LEN as u64).to_le_bytes());
        vec.extend_from_slice(&self.0);
        vec
    }
}

#[derive(Debug, thiserror::Error)]
#[error("invalid slice length for legacy node ID")]
pub struct TryFromBytesError();

impl From<TryFromSliceError> for TryFromBytesError {
    fn from(_: TryFromSliceError) -> Self {
        TryFromBytesError()
    }
}

impl<'a> TryFrom<&'a [u8]> for NodeId {
    type Error = TryFromBytesError;
    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromBytesError> {
        let bytes = bytes.try_into()?;
        Ok(NodeId(bytes))
    }
}

struct HexWrap<'a>(&'a [u8]);

impl<'a> fmt::Debug for HexWrap<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("0x")?;
        for b in self.0 {
            write!(f, "{:x}", *b)?;
        }
        Ok(())
    }
}

impl fmt::Debug for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("NodeId").field(&HexWrap(&self.0)).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let id = NodeId::generate(&mut rand::thread_rng()).unwrap();
        let v = id.encode();
        assert_eq!(v.len(), NODE_ID_LEN + 8);
        assert_eq!(v[..8], (NODE_ID_LEN as u64).to_le_bytes());
    }
}
