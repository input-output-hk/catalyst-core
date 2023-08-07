use crate::common::snapshot::result::SnapshotResult;
use crate::common::RepsVoterAssignerSource;
use chain_addr::Discrimination;
use fraction::Fraction;
use jormungandr_lib::crypto::account::Identifier;
use jormungandr_lib::interfaces::InitialUTxO;
use jormungandr_lib::interfaces::Value;
use snapshot_lib::registration::VotingRegistration;
use snapshot_lib::voting_group::VotingGroupAssigner;
use snapshot_lib::{RawSnapshot, Snapshot, VoterHIR};
use std::collections::HashSet;

pub trait SnapshotFilterSource {
    fn filter(
        &self,
        voting_threshold: Value,
        cap: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
    ) -> SnapshotFilter;
    fn filter_default(&self, reps: &HashSet<Identifier>) -> SnapshotFilter;
}

impl SnapshotFilterSource for SnapshotResult {
    fn filter(
        &self,
        voting_threshold: Value,
        cap: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
    ) -> SnapshotFilter {
        SnapshotFilter::from_snapshot_result(self, voting_threshold, cap, voting_group_assigner)
    }

    fn filter_default(&self, reps: &HashSet<Identifier>) -> SnapshotFilter {
        SnapshotFilter::from_snapshot_result_default(self, reps)
    }
}

pub struct SnapshotFilter {
    snapshot: Snapshot,
}

impl SnapshotFilter {
    pub(crate) fn from_snapshot_result_default(
        result: &SnapshotResult,
        reps: &HashSet<Identifier>,
    ) -> Self {
        Self::from_snapshot_result(
            result,
            450u64.into(),
            Fraction::new(1u64, 3u64),
            &reps.clone().into_reps_voter_assigner(),
        )
    }
}

impl SnapshotFilter {
    pub fn from_snapshot_result(
        snapshot_result: &SnapshotResult,
        voting_threshold: Value,
        cap: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
    ) -> SnapshotFilter {
        Self::from_voting_registrations(
            snapshot_result.registrations().to_vec(),
            voting_threshold,
            cap,
            voting_group_assigner,
        )
    }

    pub fn from_voting_registrations(
        voting_registrations: Vec<VotingRegistration>,
        voting_threshold: Value,
        cap: Fraction,
        voting_group_assigner: &impl VotingGroupAssigner,
    ) -> SnapshotFilter {
        Self {
            snapshot: Snapshot::from_raw_snapshot(
                RawSnapshot::from(voting_registrations),
                voting_threshold,
                cap,
                voting_group_assigner,
            )
            .unwrap(),
        }
    }

    pub fn to_voters_hirs(&self) -> Vec<VoterHIR> {
        self.snapshot
            .to_full_snapshot_info()
            .iter()
            .cloned()
            .map(|vk| vk.hir)
            .collect()
    }

    pub fn to_block0_initials(&self) -> Vec<InitialUTxO> {
        self.snapshot.to_block0_initials(Discrimination::Production)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.clone()
    }
}
