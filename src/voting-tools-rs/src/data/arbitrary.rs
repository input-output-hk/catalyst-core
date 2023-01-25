use super::RewardsAddress;
use proptest::{arbitrary::StrategyFor, prelude::*, strategy::Map};

impl Arbitrary for RewardsAddress {
    type Parameters = ();
    type Strategy = Map<StrategyFor<Vec<u8>>, fn(Vec<u8>) -> Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<Vec<u8>>().prop_map(|vec| RewardsAddress(vec.into()))
    }
}
