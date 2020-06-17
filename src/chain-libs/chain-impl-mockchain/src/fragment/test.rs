use super::*;
use crate::config::ConfigParam;
#[cfg(test)]
use quickcheck::TestResult;
use quickcheck::{Arbitrary, Gen};
use quickcheck_macros::quickcheck;

impl Arbitrary for Fragment {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.next_u32() % 13 {
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
            _ => unreachable!(),
        }
    }
}

#[quickcheck]
fn fragment_serialization_bijection(b: Fragment) -> TestResult {
    let b_got = Fragment::from_raw(&b.to_raw()).unwrap();
    TestResult::from_bool(b == b_got)
}

quickcheck! {
    fn initial_ents_serialization_bijection(config_params: ConfigParams) -> TestResult {
        chain_test_utils::property::serialization_bijection_r(config_params)
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
