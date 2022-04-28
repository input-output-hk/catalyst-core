#[cfg(feature = "serde")]
use ::serde::{Deserialize, Serialize};
use jormungandr_lib::{crypto::account::Identifier, interfaces::Value};

pub type VotingGroup = String;

/// Define High Level Intermediate Representation (HIR) for voting
/// entities in the Catalyst ecosystem.
///
/// This is intended as a high level description of the setup, which is not
/// enough on its own to spin a blockchain, but it's slimmer, easier to understand
/// and free from implementation constraints.
///
/// You can roughly read this as
/// "voting_key will participate in this voting round with role voting_group and will have voting_power influence"
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct VoterHIR {
    // Keep hex encoding as in CIP-36
    #[cfg_attr(feature = "serde", serde(with = "serde"))]
    pub voting_key: Identifier,
    /// Voting group this key belongs to.
    /// If this key belong to multiple voting groups, multiple records for the same
    /// key will be used.
    pub voting_group: VotingGroup,
    /// Voting power as processed by the snapshot
    pub voting_power: Value,
}

#[cfg(feature = "serde")]
mod serde {
    use super::*;
    use ::serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(voting_key: &Identifier, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&voting_key.to_hex())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Identifier, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        Identifier::from_hex(hex.trim_start_matches("0x"))
            .map_err(|e| D::Error::custom(format!("invalid public key: {}", e)))
    }
}
