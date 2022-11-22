use std::convert::TryInto;

use chain_impl_mockchain::testing::TestGen;
use jormungandr_lib::interfaces::Value;
use rand::Rng;
use snapshot_lib::registration::{Delegations, VotingRegistration};
use snapshot_lib::{voting_group::RepsVotersAssigner, Error, Snapshot};
use snapshot_lib::{
    voting_group::{DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    CATALYST_VOTING_PURPOSE_TAG,
};
use snapshot_lib::{Fraction, SnapshotInfo};
use time::OffsetDateTime;
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;

#[derive(Debug, Clone)]
pub struct RawSnapshot {
    pub tag: String,
    pub content: RawSnapshotInput,
}

impl Default for RawSnapshot {
    fn default() -> RawSnapshot {
        RawSnapshotBuilder::default().build()
    }
}

impl RawSnapshot {
    pub fn empty(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            content: RawSnapshotInput {
                snapshot: Vec::new().into(),
                update_timestamp: 0,
                min_stake_threshold: 0.into(),
                voting_power_cap: 0.into(),
                direct_voters_group: None,
                representatives_group: None,
            },
        }
    }
}

pub trait RawSnapshotExtension {
    fn into_full_snapshot_infos(
        self,
        assigner: &RepsVotersAssigner,
    ) -> Result<Vec<SnapshotInfo>, Error>;
}

impl RawSnapshotExtension for RawSnapshot {
    fn into_full_snapshot_infos(
        self,
        assigner: &RepsVotersAssigner,
    ) -> Result<Vec<SnapshotInfo>, Error> {
        Ok(Snapshot::from_raw_snapshot(
            self.content.snapshot,
            self.content.min_stake_threshold,
            self.content.voting_power_cap,
            assigner,
        )?
        .to_full_snapshot_info())
    }
}

impl From<RawSnapshot> for RepsVotersAssigner {
    fn from(raw_snapshot: RawSnapshot) -> Self {
        let direct_voter = raw_snapshot
            .content
            .direct_voters_group
            .unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.into());
        let representative = raw_snapshot
            .content
            .representatives_group
            .unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.into());

        RepsVotersAssigner::new(direct_voter, representative)
    }
}

impl TryInto<Vec<SnapshotInfo>> for RawSnapshot {
    type Error = Error;
    fn try_into(self) -> Result<Vec<SnapshotInfo>, Self::Error> {
        let assigner = self.clone().into();
        self.into_full_snapshot_infos(&assigner)
    }
}

#[derive(Debug)]
pub struct RawSnapshotBuilder {
    tag: String,
    update_timestamp: i64,
    min_stake_threshold: Value,
    voting_power_cap: Fraction,
    direct_voters_group: Option<String>,
    representatives_group: Option<String>,
    voting_registrations_count: usize,
    voting_registrations: Option<Vec<VotingRegistration>>,
}

impl Default for RawSnapshotBuilder {
    fn default() -> RawSnapshotBuilder {
        Self {
            tag: "daily".to_string(),
            update_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
            min_stake_threshold: 0.into(),
            voting_power_cap: 100.into(),
            direct_voters_group: Some(DEFAULT_DIRECT_VOTER_GROUP.into()),
            representatives_group: Some(DEFAULT_REPRESENTATIVE_GROUP.into()),
            voting_registrations_count: 2,
            voting_registrations: None,
        }
    }
}

impl RawSnapshotBuilder {
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn with_voting_registrations_count(mut self, voting_registrations_count: usize) -> Self {
        self.voting_registrations_count = voting_registrations_count;
        self
    }

    pub fn with_voting_registrations(
        mut self,
        voting_registrations: Vec<VotingRegistration>,
    ) -> Self {
        self.voting_registrations = Some(voting_registrations);
        self
    }

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.update_timestamp = timestamp;
        self
    }

    pub fn with_voting_power_cap(mut self, voting_power_cap: Fraction) -> Self {
        self.voting_power_cap = voting_power_cap;
        self
    }

    pub fn with_min_stake_threshold(mut self, min_stake_threshold: Value) -> Self {
        self.min_stake_threshold = min_stake_threshold;
        self
    }

    pub fn build(mut self) -> RawSnapshot {
        let mut rng = rand::rngs::OsRng;
        let mut delegation_type_count = 0;

        if self.voting_registrations.is_none() {
            self.voting_registrations = Some(
                std::iter::from_fn(|| {
                    Some(VotingRegistration {
                        stake_public_key: TestGen::public_key().to_string(),
                        voting_power: rng.gen_range(1u64, 1_00u64).into(),
                        reward_address: TestGen::public_key().to_string(),
                        delegations: if delegation_type_count > self.voting_registrations_count / 2
                        {
                            delegation_type_count += 1;
                            Delegations::New(vec![
                                (TestGen::identifier().into(), 1),
                                (TestGen::identifier().into(), 2),
                            ])
                        } else {
                            delegation_type_count += 1;
                            Delegations::Legacy(TestGen::identifier().into())
                        },
                        voting_purpose: CATALYST_VOTING_PURPOSE_TAG,
                    })
                })
                .take(self.voting_registrations_count)
                .collect::<Vec<_>>(),
            )
        }

        RawSnapshot {
            content: RawSnapshotInput {
                snapshot: self.voting_registrations.unwrap().into(),
                update_timestamp: self.update_timestamp,
                min_stake_threshold: self.min_stake_threshold,
                voting_power_cap: self.voting_power_cap,
                direct_voters_group: self.direct_voters_group,
                representatives_group: self.representatives_group,
            },
            tag: self.tag,
        }
    }
}

#[derive(Debug)]
pub struct RawSnapshotUpdater {
    raw_snapshot: RawSnapshot,
}

impl From<RawSnapshot> for RawSnapshotUpdater {
    fn from(raw_snapshot: RawSnapshot) -> Self {
        Self { raw_snapshot }
    }
}

impl RawSnapshotUpdater {
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.raw_snapshot.tag = tag.into();
        self
    }

    pub fn build(self) -> RawSnapshot {
        self.raw_snapshot
    }
}
