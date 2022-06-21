use super::*;
use ::proptest::{prelude::*, strategy::BoxedStrategy};
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
