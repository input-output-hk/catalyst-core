//! Support for legacy features.

use rand_core::RngCore;

const NODE_ID_LEN: usize = 24;

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
        let mut vec = Vec::with_capacity(NODE_ID_LEN + 8);
        vec.extend_from_slice(&(NODE_ID_LEN as u64).to_le_bytes());
        vec.extend_from_slice(&self.0);
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
