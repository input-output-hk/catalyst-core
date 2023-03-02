#![allow(dead_code)]

use crate::data::SnapshotEntry;
use crate::VotingPowerSource;
use bigdecimal::ToPrimitive;
use cardano_serialization_lib::address::RewardAddress;
use cardano_serialization_lib::crypto::PublicKey;

/// Allows [`SnapshotEntry`] struct to be assertable
pub trait VerifiableSnapshotOutput {
    /// returns assertion struct
    fn assert(&self) -> SnapshotOutputAssert;
}

impl VerifiableSnapshotOutput for SnapshotEntry {
    fn assert(&self) -> SnapshotOutputAssert {
        SnapshotOutputAssert::new(self)
    }
}

/// Fluent api for [`SnapshotEntry`] assertions
pub struct SnapshotOutputAssert<'a> {
    output: &'a SnapshotEntry,
}

impl<'a> SnapshotOutputAssert<'a> {
    /// Creates new instance based on [`SnapshotEntry`] reference
    #[must_use]
    pub fn new(output: &'a SnapshotEntry) -> Self {
        Self { output }
    }

    /// Asserts expected voting power field from [`SnapshotEntry`]
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

    /// Asserts delegations address field from [`SnapshotEntry`]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn voting_power_source(&self, voting_power_source: &VotingPowerSource) {
        assert_eq!(
            voting_power_source, &self.output.voting_power_source,
            "delegation target"
        );
    }

    /// Asserts reward address field from [`SnapshotEntry`]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn reward_address(&self, reward_address: &RewardAddress) {
        assert_eq!(
            reward_address.to_address().to_hex(),
            hex::encode(&self.output.rewards_address.0),
            "different rewards address"
        );
    }

    /// Asserts stake public key field from [`SnapshotEntry`]
    /// # Panics
    ///
    /// Panics on assertion failed
    pub fn stake_key(&self, public_key: &PublicKey) {
        assert_eq!(
            &public_key.to_hex(),
            &hex::encode(self.output.stake_key.clone()),
            "different stake public key"
        );
    }
}
