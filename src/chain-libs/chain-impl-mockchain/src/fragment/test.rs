use super::*;
use crate::config::ConfigParam;
#[cfg(test)]
use crate::testing::serialization::serialization_bijection;
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

impl Arbitrary for Fragment {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let r = g.next_u32() % 14;
        match r {
            0 => Fragment::Initial(Arbitrary::arbitrary(g)),
            1 => Fragment::OldUtxoDeclaration(Arbitrary::arbitrary(g)),
            2 => Fragment::Transaction(Arbitrary::arbitrary(g)),
            3 => Fragment::OwnerStakeDelegation(Arbitrary::arbitrary(g)),
            4 => Fragment::StakeDelegation(Arbitrary::arbitrary(g)),
            5 => Fragment::PoolRegistration(Arbitrary::arbitrary(g)),
            6 => Fragment::PoolRetirement(Arbitrary::arbitrary(g)),
            7 => Fragment::PoolUpdate(Arbitrary::arbitrary(g)),
            8 => Fragment::UpdateProposal(Arbitrary::arbitrary(g)),
            9 => Fragment::UpdateVote(Arbitrary::arbitrary(g)),
            10 => Fragment::VotePlan(Arbitrary::arbitrary(g)),
            11 => Fragment::VoteCast(Arbitrary::arbitrary(g)),
            12 => Fragment::VoteTally(Arbitrary::arbitrary(g)),
            13 => Fragment::MintToken(Arbitrary::arbitrary(g)),
            _ => unreachable!(),
        }
    }
}

impl Arbitrary for ConfigParams {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let size = u8::arbitrary(g) as usize;
        ConfigParams(
            std::iter::repeat_with(|| ConfigParam::arbitrary(g))
                .take(size)
                .collect(),
        )
    }
}

quickcheck! {
    fn fragment_serialization_bijection(b: Fragment) -> TestResult {
        serialization_bijection(b)
    }

    fn initial_ents_serialization_bijection(config_params: ConfigParams) -> TestResult {
        serialization_bijection(config_params)
    }
}
