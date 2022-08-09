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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq)]
pub struct VoterHIR {
    // Keep hex encoding as in CIP-36
    #[serde(with = "serde")]
    pub voting_key: Identifier,
    /// Voting group this key belongs to.
    /// If this key belong to multiple voting groups, multiple records for the same
    /// key will be used.
    pub voting_group: VotingGroup,
    /// Voting power as processed by the snapshot
    pub voting_power: Value,
}

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

#[cfg(any(test, feature = "proptest"))]
pub mod tests {
    use super::*;
    use ::proptest::{prelude::*, strategy::BoxedStrategy};
    use jormungandr_lib::crypto::account::Identifier;
    use std::ops::Range;

    impl Arbitrary for VoterHIR {
        type Parameters = (String, VpRange);
        type Strategy = BoxedStrategy<Self>;
        fn arbitrary_with(args: Self::Parameters) -> Self::Strategy {
            (any::<([u8; 32])>(), args.1 .0)
                .prop_map(move |(key, voting_power)| VoterHIR {
                    voting_key: Identifier::from_hex(&hex::encode(key)).unwrap(),
                    voting_power: voting_power.into(),
                    voting_group: args.0.clone(),
                })
                .boxed()
        }
    }

    pub struct VpRange(Range<u64>);

    impl VpRange {
        pub const fn ada_distribution() -> Self {
            Self(1..45_000_000_000)
        }
    }

    impl Default for VpRange {
        fn default() -> Self {
            Self(0..u64::MAX)
        }
    }

    impl From<Range<u64>> for VpRange {
        fn from(range: Range<u64>) -> Self {
            Self(range)
        }
    }
}
