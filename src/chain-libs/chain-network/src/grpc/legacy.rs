//! Support for legacy features.

use rand_core::RngCore;
use std::convert::TryInto;

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
        config.limit(NODE_ID_LEN.try_into().unwrap());

        let mut vec = Vec::with_capacity(NODE_ID_LEN);
        config.serialize_into(&mut vec, &self.0).unwrap();
        vec
    }
}
