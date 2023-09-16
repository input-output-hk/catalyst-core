use ::serde::{Deserialize, Serialize};
use jormungandr_lib::{
    crypto::account::{Identifier, SigningKey},
    interfaces::Address,
    interfaces::Value,
};

pub type VotingGroup = String;

fn is_false(b: &bool) -> bool {
    *b == false
}

/// Define High Level Intermediate Representation (HIR) for voting
/// entities in the Catalyst ecosystem.
///
/// This is intended as a high level description of the setup, which is not
/// enough on its own to spin a blockchain, but it's slimmer, easier to understand
/// and free from implementation constraints.
///
/// You can roughly read this as
/// "`voting_key` will participate in this voting round with role voting_group and will have voting_power influence"
#[derive(Serialize, Deserialize, Debug, Clone /*, PartialEq, Hash, Eq*/)]
pub struct VoterHIR {
    // Keep hex encoding as in CIP-36
    #[serde(with = "serde")]
    pub voting_key: Identifier,
    // Jormungandr chain address
    pub address: Address,

    /// Voting group this key belongs to.
    /// If this key belong to multiple voting groups, multiple records for the same
    /// key will be used.
    pub voting_group: VotingGroup,
    /// Voting power as processed by the snapshot
    pub voting_power: Value,

    /// Under threshold (voter doesn't have enough voting power)
    /// if `true` this voter can not participate in voting.
    #[serde(default, skip_serializing_if = "is_false")]
    pub underthreshold: bool,

    /// Overlimit (voter is max voting power threshold limited)
    /// This field is just an indication and doesn't affect ability to vote.
    #[serde(default, skip_serializing_if = "is_false")]
    pub overlimit: bool,

    /// PrivateKey is only created when making a loadtest snapshot.
    /// Its ONLY used for fake accounts used in the load test, and
    /// can not be created or derived for a legitimate voter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub private_key: Option<String>,
}

impl VoterHIR {
    #[must_use]
    pub fn to_loadtest_snapshot(&self) -> Self {
        // Generate a new Voting Key pair.
        let loadtest_key = SigningKey::generate(rand::thread_rng());
        let loadtest_voting_key = loadtest_key.identifier();

        let discriminator = self.address.1.discrimination();
        let loadtest_address = loadtest_voting_key.to_address(discriminator);

        let private_key = format!("0x{}", loadtest_key.to_hex());

        Self {
            voting_key: loadtest_voting_key,
            address: loadtest_address.into(),
            voting_group: self.voting_group.clone(),
            voting_power: self.voting_power.into(),
            underthreshold: self.underthreshold,
            overlimit: self.overlimit,
            private_key: Some(private_key),
        }
    }

    #[must_use]
    pub fn cap_voting_power(&self, cap: u64) -> Self {
        let mut voting_power = self.voting_power.as_u64();
        let mut overlimit = self.overlimit;
        if voting_power > cap {
            voting_power = cap;
            overlimit = true;
        };

        VoterHIR {
            voting_key: self.voting_key.clone(),
            address: self.address.clone(),
            voting_group: self.voting_group.clone(),
            voting_power: voting_power.into(),
            underthreshold: self.underthreshold,
            overlimit, // Set overlimit if required.
            private_key: self.private_key.clone(),
        }
    }
}

mod serde {
    use super::*;
    use ::serde::{de::Error, Deserializer, Serializer};

    pub fn serialize<S>(voting_key: &Identifier, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", voting_key.to_hex()))
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
            (any::<[u8; 32]>(), args.1 .0)
                .prop_map(move |(key, voting_power)| VoterHIR {
                    // Two representations of exactly the same key.
                    voting_key: Identifier::from_hex(&hex::encode(key)).unwrap(),
                    address: chain_addr::Address(
                        chain_addr::Discrimination::Production,
                        chain_addr::Kind::Account(
                            Identifier::from_hex(&hex::encode(key))
                                .unwrap()
                                .to_inner()
                                .into(),
                        ),
                    )
                    .into(),
                    voting_power: voting_power.into(),
                    voting_group: args.0.clone(),
                    underthreshold: false,
                    overlimit: false,
                    private_key: None,
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
