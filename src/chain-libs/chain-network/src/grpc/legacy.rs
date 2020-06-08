//! Support for legacy features.

use rand_core::RngCore;

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

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
