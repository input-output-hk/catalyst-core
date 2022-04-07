use crate::tokens::policy_hash::{PolicyHash, POLICY_HASH_SIZE};
#[cfg(any(test, feature = "property-test-api"))]
use proptest::prelude::*;

use chain_core::{
    packer::Codec,
    property::{Deserialize, ReadError, Serialize, WriteError},
};
use cryptoxide::{blake2b::Blake2b, digest::Digest};
use thiserror::Error;
use typed_bytes::ByteBuilder;

/// A minting policy consists of multiple entries defining different
/// constraints on the minting process. An empty policy means that new tokens
/// cannot be minted during the chain run.
///
/// Minting policies are meant to be ignored in block0 fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(test, feature = "property-test-api"),
    derive(test_strategy::Arbitrary)
)]
pub struct MintingPolicy(
    #[cfg_attr(
        any(test, feature = "property-test-api"),
        strategy(Just(vec![]))
    )]
    Vec<MintingPolicyEntry>,
);

/// An entry of a minting policy. Currently there are no entries available.
/// This is reserved for the future use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MintingPolicyEntry {}

/// Error while checking a minting transaction against the current system state.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MintingPolicyViolation {
    #[error("the policy of this token does not allow minting")]
    AdditionalMintingNotAllowed,
}

impl MintingPolicy {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn check_minting_tx(&self) -> Result<(), MintingPolicyViolation> {
        if self.0.is_empty() {
            return Err(MintingPolicyViolation::AdditionalMintingNotAllowed);
        }

        for _entry in &self.0 {
            unreachable!("implement this when we have actual minting policies");
        }

        Ok(())
    }

    pub fn entries(&self) -> &[MintingPolicyEntry] {
        &self.0
    }

    pub fn bytes(&self) -> Vec<u8> {
        let bb: ByteBuilder<Self> = ByteBuilder::new();
        bb.u8(0).finalize_as_vec()
    }

    pub fn hash(&self) -> PolicyHash {
        let mut result = [0u8; POLICY_HASH_SIZE];
        if !self.0.is_empty() {
            let mut hasher = Blake2b::new(POLICY_HASH_SIZE);
            hasher.input(&self.bytes());
            hasher.result(&mut result);
        }
        result.into()
    }
}

impl Default for MintingPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for MintingPolicy {
    fn serialize<W: std::io::Write>(&self, codec: &mut Codec<W>) -> Result<(), WriteError> {
        codec.put_u8(0_u8)
    }
}

impl Deserialize for MintingPolicy {
    fn deserialize<R: std::io::Read>(codec: &mut Codec<R>) -> Result<Self, ReadError> {
        let no_entries = codec.get_u8()?;
        if no_entries != 0 {
            return Err(ReadError::InvalidData(
                "non-zero number of minting policy entries, but they are currently unimplemented"
                    .to_string(),
            ));
        }
        Ok(Self::new())
    }
}

#[cfg(any(test, feature = "property-test-api"))]
mod tests {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for MintingPolicy {
        fn arbitrary<G: Gen>(_g: &mut G) -> Self {
            Self::new()
        }
    }
}
