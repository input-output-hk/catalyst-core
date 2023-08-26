use chain_impl_mockchain::testing::TestGen;
use itertools::Itertools;
use jormungandr_lib::crypto::account::Identifier;
use rand::Rng;
use serde::{Deserialize, Serialize};
use snapshot_lib::{
    registration::{RewardAddress, StakeAddress},
    KeyContribution, SnapshotInfo, VoterHIR,
};
use time::OffsetDateTime;
use vit_servicing_station_lib::v0::endpoints::snapshot::SnapshotInfoInput;

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tag: String,
    pub content: SnapshotInfoInput,
}

impl Default for Snapshot {
    fn default() -> Snapshot {
        SnapshotBuilder::default().build()
    }
}

#[derive(Debug)]
pub struct SnapshotBuilder {
    tag: String,
    groups: Vec<String>,
    voters_count: usize,
    contributions_count: usize,
    update_timestamp: i64,
}

impl Default for SnapshotBuilder {
    fn default() -> SnapshotBuilder {
        Self {
            tag: "daily".to_string(),
            groups: vec!["direct".to_string(), "dreps".to_string()],
            voters_count: 3,
            contributions_count: 5,
            update_timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }
}

impl SnapshotBuilder {
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn with_entries_count(mut self, voters_count: usize) -> Self {
        self.voters_count = voters_count;
        self
    }

    pub fn with_contributions_count(mut self, contributions_count: usize) -> Self {
        self.contributions_count = contributions_count;
        self
    }

    pub fn with_groups<S: Into<String>>(mut self, groups: Vec<S>) -> Self {
        self.groups = groups.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_timestamp(mut self, timestamp: i64) -> Self {
        self.update_timestamp = timestamp;
        self
    }

    pub fn build(self) -> Snapshot {
        let mut rng = rand::rngs::OsRng;

        let voters_count = {
            if self.voters_count == 0 {
                rng.gen_range(1usize, 1_000usize)
            } else {
                self.voters_count
            }
        };

        Snapshot {
            tag: self.tag.clone(),
            content: SnapshotInfoInput {
                snapshot: std::iter::from_fn(|| {
                    Some(SnapshotInfo {
                        contributions: std::iter::from_fn(|| {
                            Some(KeyContribution {
                                reward_address: RewardAddress(format!(
                                    "address_{:?}",
                                    rng.gen_range(1u64, 1_000u64)
                                )),
                                stake_public_key: StakeAddress(format!(
                                    "address_{:?}",
                                    rng.gen_range(1u64, 1_000u64)
                                )),
                                value: rng.gen_range(1u64, 1_000u64),
                            })
                        })
                        .take(self.contributions_count)
                        .collect(),
                        hir: {
                            let identifier = TestGen::identifier();

                            VoterHIR {
                                voting_key: identifier.clone().into(),
                                voting_group: self.groups[rng.gen_range(0, self.groups.len())]
                                    .to_string(),
                                voting_power: rng.gen_range(1u64, 1_000u64).into(),
                                address: chain_addr::Address(
                                    chain_addr::Discrimination::Production,
                                    chain_addr::Kind::Account(identifier.into()),
                                )
                                .into(),
                            }
                        },
                    })
                })
                .take(voters_count)
                .collect(),
                update_timestamp: self.update_timestamp,
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct VoterInfo {
    #[serde(
        deserialize_with = "vit_servicing_station_lib::utils::serde::deserialize_unix_timestamp_from_rfc3339"
    )]
    #[serde(
        serialize_with = "vit_servicing_station_lib::utils::serde::serialize_unix_timestamp_as_rfc3339"
    )]
    pub last_updated: i64,
    pub voter_info: Vec<VotingPower>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VotingPower {
    pub voting_power: u64,
    pub voting_group: String,
    pub delegations_power: u64,
    pub delegations_count: u64,
    pub voting_power_saturation: f64,
}

impl PartialEq for VotingPower {
    fn eq(&self, other: &Self) -> bool {
        self.voting_power == other.voting_power
            && self.voting_group == other.voting_group
            && self.delegations_power == other.delegations_power
            && self.delegations_count == other.delegations_count
        //&& self.voting_power_saturation == other.voting_power_saturation
    }
}

impl Eq for VotingPower {}

impl From<SnapshotInfo> for VotingPower {
    fn from(snapshot_info: SnapshotInfo) -> Self {
        let delegations_power: u64 = snapshot_info
            .contributions
            .iter()
            .map(|KeyContribution { value, .. }| value)
            .sum();
        Self {
            voting_power: snapshot_info.hir.voting_power.into(),
            voting_group: snapshot_info.hir.voting_group,
            delegations_power,
            delegations_count: snapshot_info.contributions.len() as u64,
            voting_power_saturation: 0 as f64,
        }
    }
}

#[derive(Debug)]
pub struct SnapshotUpdater {
    snapshot: Snapshot,
}

impl From<Snapshot> for SnapshotUpdater {
    fn from(snapshot: Snapshot) -> Self {
        Self { snapshot }
    }
}

impl SnapshotUpdater {
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.snapshot.tag = tag.into();
        self
    }

    pub fn add_new_arbitrary_voters(mut self) -> Self {
        let extra_snapshot = SnapshotBuilder::default()
            .with_groups(
                self.snapshot
                    .content
                    .snapshot
                    .iter()
                    .map(|x| x.hir.voting_group.clone())
                    .unique()
                    .collect(),
            )
            .build();

        self.snapshot
            .content
            .snapshot
            .extend(extra_snapshot.content.snapshot.iter().cloned());
        self
    }

    pub fn add_contributions_to_voter(
        mut self,
        contributions: Vec<KeyContribution>,
        voting_key: &Identifier,
    ) -> Self {
        let voter = self
            .snapshot
            .content
            .snapshot
            .iter_mut()
            .find(|entry| entry.hir.voting_key == *voting_key);
        if let Some(voter) = voter {
            voter.contributions.extend(contributions)
        }
        self
    }

    pub fn update_voting_power(mut self) -> Self {
        let mut rng = rand::rngs::OsRng;
        for entry in self.snapshot.content.snapshot.iter_mut() {
            let mut voting_power: u64 = entry.hir.voting_power.into();
            voting_power += rng.gen_range(1u64, 1_000u64);
            entry.hir.voting_power = voting_power.into();
        }
        self
    }

    pub fn build(self) -> Snapshot {
        self.snapshot
    }
}
