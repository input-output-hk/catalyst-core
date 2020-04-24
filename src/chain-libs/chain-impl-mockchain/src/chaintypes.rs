//! Shared Types between ledger and chain
//!
//! Common types that are found in the header but are used or stored in the
//! ledger for either validation or some state tracking

use crate::key::Hash;
use strum_macros::{Display, EnumString, IntoStaticStr};

pub type HeaderId = Hash; // TODO: change to DigestOf<Blake2b256, Header>

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChainLength(pub(crate) u32);

impl From<u32> for ChainLength {
    fn from(n: u32) -> ChainLength {
        ChainLength(n)
    }
}

impl From<ChainLength> for u32 {
    fn from(chain_length: ChainLength) -> u32 {
        chain_length.0
    }
}

impl ChainLength {
    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn increase(self) -> Self {
        ChainLength(self.0.checked_add(1).unwrap())
    }

    #[inline]
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn nth_ancestor(self, depth: u32) -> Option<ChainLength> {
        self.0.checked_sub(depth).map(ChainLength)
    }
}

impl std::fmt::Display for ChainLength {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, Display, EnumString, IntoStaticStr, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum ConsensusType {
    #[strum(to_string = "bft")]
    Bft = 1,
    #[strum(to_string = "genesis")]
    GenesisPraos = 2,
}

// alias for old name
pub type ConsensusVersion = ConsensusType;

impl ConsensusType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(ConsensusType::Bft),
            2 => Some(ConsensusType::GenesisPraos),
            _ => None,
        }
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for ConsensusType {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ConsensusType::from_u16(u16::arbitrary(g) % 2 + 1).unwrap()
        }
    }

    impl Arbitrary for ChainLength {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            ChainLength(Arbitrary::arbitrary(g))
        }
    }
}
