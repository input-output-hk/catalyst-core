use chain_impl_mockchain::testing::TestGen;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use snapshot_lib::{KeyContribution, SnapshotInfo, VoterHIR};
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tag: String,
    pub content: Vec<SnapshotInfo>,
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
}

impl Default for SnapshotBuilder {
    fn default() -> SnapshotBuilder {
        Self {
            tag: "daily".to_string(),
            groups: vec!["direct".to_string(), "dreps".to_string()],
            voters_count: 3,
            contributions_count: 5,
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
            content: std::iter::from_fn(|| {
                Some(SnapshotInfo {
                    contributions: std::iter::from_fn(|| {
                        Some(KeyContribution {
                            reward_address: format!("address_{:?}", rng.gen_range(1u64, 1_000u64)),
                            value: rng.gen_range(1u64, 1_000u64),
                        })
                    })
                    .take(self.contributions_count)
                    .collect(),
                    hir: VoterHIR {
                        voting_key: TestGen::identifier().into(),
                        voting_group: self.groups[rng.gen_range(0, self.groups.len())].to_string(),
                        voting_power: rng.gen_range(1u64, 1_000u64).into(),
                    },
                })
            })
            .take(voters_count)
            .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct VoterInfo {
    pub voting_power: u64,
    pub voting_group: String,
    pub delegations_power: u64,
    pub delegations_count: u64,
}

impl From<SnapshotInfo> for VoterInfo {
    fn from(snapshot_info: SnapshotInfo) -> Self {
        let delegations_power: u64 = snapshot_info
            .contributions
            .iter()
            .map(
                |KeyContribution {
                     reward_address: _,
                     value,
                 }| value,
            )
            .sum();
        Self {
            voting_power: snapshot_info.hir.voting_power.into(),
            voting_group: snapshot_info.hir.voting_group,
            delegations_power,
            delegations_count: snapshot_info.contributions.len() as u64,
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
                    .iter()
                    .map(|x| x.hir.voting_group.clone())
                    .unique()
                    .collect(),
            )
            .build();

        self.snapshot
            .content
            .extend(extra_snapshot.content.iter().cloned());
        self
    }

    pub fn update_voting_power(mut self) -> Self {
        let mut rng = rand::rngs::OsRng;
        for entry in self.snapshot.content.iter_mut() {
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
