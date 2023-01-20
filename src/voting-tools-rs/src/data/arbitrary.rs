use super::*;
use proptest::{arbitrary::StrategyFor, prelude::*, strategy::Map};

/// A registration with the private key for the stake key
struct RegistrationAndKey {
    pub registration: Registration,
}

/// Inputs to the strategy for generating arbitrary registrations
type Inputs = (RewardsAddress, Nonce);

impl Arbitrary for SignedRegistration {
    type Parameters = ();
    type Strategy = Map<StrategyFor<Inputs>, fn(Inputs) -> Self>;
    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        any::<Inputs>().prop_map(|(rewards_address, nonce)| {
            generate_signed_registration(rewards_address, nonce)
        })
    }
}

fn generate_signed_registration(
    rewards_address: RewardsAddress,
    nonce: Nonce,
) -> SignedRegistration {
    todo!()
}
