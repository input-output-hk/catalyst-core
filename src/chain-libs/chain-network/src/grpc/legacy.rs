//! Support for legacy features.

use rand_core::RngCore;
use std::convert::TryFrom;

const NODE_ID_LEN: usize = 32;

/// Represents a randomly generated node ID such as was present in subscription
/// requests and responses in Jormungandr versions prior to 0.9
/// (as implemented in the old `network-grpc` crate).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId([u8; NODE_ID_LEN]);

impl NodeId {
    pub fn generate(rng: &mut impl RngCore) -> Result<Self, rand_core::Error> {
        let mut bytes = [0; NODE_ID_LEN];
        rng.try_fill_bytes(&mut bytes)?;
        Ok(NodeId(bytes))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut config = bincode::config();
        config.limit(u64::try_from(NODE_ID_LEN).unwrap());

        let mut vec = Vec::with_capacity(NODE_ID_LEN);
        config.serialize_into(&mut vec, &self.0).unwrap();
        vec
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
