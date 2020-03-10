#[warn(unused_imports)]
use super::{AccountState, DelegationType, LastRewards, SpendingCounter};
use quickcheck::{Arbitrary, Gen};

impl Arbitrary for SpendingCounter {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        u32::arbitrary(gen).into()
    }
}

impl Arbitrary for AccountState<()> {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        AccountState {
            counter: Arbitrary::arbitrary(gen),
            delegation: DelegationType::Full(Arbitrary::arbitrary(gen)),
            value: Arbitrary::arbitrary(gen),
            last_rewards: LastRewards::default(),
            extra: (),
        }
    }
}
