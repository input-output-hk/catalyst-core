#[warn(unused_imports)]
use super::{
    AccountState, DelegationType, LastRewards, SpendingCounter, SpendingCounterIncreasing,
};
use imhamt::Hamt;
use quickcheck::{Arbitrary, Gen};

impl Arbitrary for SpendingCounter {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        u32::arbitrary(gen).into()
    }
}

impl Arbitrary for SpendingCounterIncreasing {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        SpendingCounterIncreasing::new_from_counter(SpendingCounter::arbitrary(gen))
    }
}

impl Arbitrary for AccountState<()> {
    fn arbitrary<G: Gen>(gen: &mut G) -> Self {
        AccountState {
            spending: Arbitrary::arbitrary(gen),
            delegation: DelegationType::Full(Arbitrary::arbitrary(gen)),
            value: Arbitrary::arbitrary(gen),
            tokens: Hamt::new(),
            last_rewards: LastRewards::default(),
            #[cfg(feature = "evm")]
            evm_state: chain_evm::state::AccountState::default(),
            extra: (),
        }
    }
}
