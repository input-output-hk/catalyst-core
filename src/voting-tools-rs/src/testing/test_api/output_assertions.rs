#![allow(dead_code)]

use crate::model::Output;
use bigdecimal::ToPrimitive;
use cardano_serialization_lib::address::RewardAddress;
use cardano_serialization_lib::crypto::PublicKey;

/// Allows [Output] struct to be assertable
pub trait VerifiableSnapshotOutput {
    /// returns assertion struct
    fn assert(&self) -> SnapshotOutputAssert;
}

impl VerifiableSnapshotOutput for Output {
    fn assert(&self) -> SnapshotOutputAssert {
        SnapshotOutputAssert::new(self)
    }
}

/// Fluent api for [Output] assertions
pub struct SnapshotOutputAssert<'a> {
    output: &'a Output,
}

impl<'a> SnapshotOutputAssert<'a> {
    /// Creates new instance based on [Output] reference
    #[must_use]
    pub fn new(output: &'a Output) -> Self {
        Self { output }
    }

    /// Asserts expected voting power field from [Output]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn voting_power(&self, voting_power: u64) {
        assert_eq!(
            voting_power,
            self.output
                .voting_power
                .to_u64()
                .expect("cannot convert voting power to u64"),
            "wrong voting power"
        );
    }

    /// Asserts reward address field from [Output]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn reward_address(&self, reward_address: &RewardAddress) {
        assert_eq!(
            reward_address.to_address().to_hex(),
            self.output.rewards_address.to_string(),
            "different rewards address"
        );
    }

    /// Asserts stake public key field from [Output]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn stake_key(&self, public_key: &PublicKey) {
        assert_eq!(
            &public_key.to_hex(),
            &self.output.stake_public_key.to_string(),
            "different stake public key"
        );
    }
}
