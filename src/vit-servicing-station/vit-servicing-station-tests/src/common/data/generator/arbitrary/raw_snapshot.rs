use std::iter;

use chain_impl_mockchain::testing::TestGen;
use snapshot_lib::{
    registration::{Delegations, Delegations::*, VotingRegistration},
    RawSnapshot,
};

use super::ArbitraryGenerator;

const LEGACY_DELEGATION: usize = 1;
const NEW_DELEGATION: usize = 0;

#[derive(Clone)]
pub struct ArbitraryRawSnapshotGenerator {
    id_generator: ArbitraryGenerator,
}

impl Default for ArbitraryRawSnapshotGenerator {
    fn default() -> Self {
        Self {
            id_generator: ArbitraryGenerator::new(),
        }
    }
}

impl ArbitraryRawSnapshotGenerator {
    pub fn legacy_delegation(&mut self) -> Delegations {
        Delegations::Legacy(TestGen::identifier().into())
    }

    pub fn new_delegation(&mut self) -> Delegations {
        let size = self.id_generator.random_size();
        Delegations::New(
            iter::from_fn(|| Some((TestGen::identifier().into(), self.id_generator.next_u32())))
                .take(size)
                .collect(),
        )
    }

    pub fn delegation(&mut self) -> Delegations {
        let delegation_type = self.id_generator.random_index(1);
        match delegation_type {
            LEGACY_DELEGATION => self.legacy_delegation(),
            NEW_DELEGATION => self.new_delegation(),
        }
    }

    pub fn voting_registration(&mut self) -> VotingRegistration {
        VotingRegistration {
            stake_public_key: chain_impl_mockchain::testing::TestGen::public_key().to_string(),
            voting_power: self.id_generator.next_u64().into(),
            reward_address: chain_impl_mockchain::testing::TestGen::public_key().to_string(),
            delegations: self.delegation(),
            voting_purpose: self.id_generator.next_u64(),
        }
    }

    pub fn raw_snapshot(&mut self) -> RawSnapshot {
        let size = self.id_generator.random_size();
        iter::from_fn(|| Some(self.voting_registration()))
            .take(size)
            .collect::<Vec<_>>()
            .into()
    }
}
