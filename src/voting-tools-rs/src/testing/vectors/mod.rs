use cardano_serialization_lib::chain_crypto::ed25519;
use rand::{Rng, RngCore, CryptoRng};

/// A struct which wraps an RNG and declares that it is cryptographically secure
///
/// Warning - this will be accepted in cryptographically sensitive contexts, regardless of the
/// security properties of the underlying RNG
///
/// TLDR: don't use this in prod code
struct DangerCryptoRng<T: RngCore> {
    inner: T,
}

impl<T: RngCore> CryptoRng for DangerCryptoRng<T> {}

use crate::model::{Registration, StakeVKey};

pub fn generate_stake_key(rng: &mut impl Rng + CryptoRng) -> StakeVKey {
    cardano_serialization_lib::StakeDelegation
}

pub fn generate_registration(rng: &mut impl Rng + CryptoRng) -> Registration {
    
}
