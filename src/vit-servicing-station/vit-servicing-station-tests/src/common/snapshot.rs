use chain_impl_mockchain::testing::TestGen;
use itertools::Itertools;
use rand::Rng;
use serde::{Deserialize, Serialize};
use voting_hir::VoterHIR;
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub tag: String,
    pub content: Vec<VoterHIR>,
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
    count: usize,
}

impl Default for SnapshotBuilder {
    fn default() -> SnapshotBuilder {
        Self {
            tag: "daily".to_string(),
            groups: vec!["direct".to_string(), "dreps".to_string()],
            count: 3,
        }
    }
}

impl SnapshotBuilder {
    pub fn with_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.tag = tag.into();
        self
    }

    pub fn with_entries_count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    pub fn with_groups<S: Into<String>>(mut self, groups: Vec<S>) -> Self {
        self.groups = groups.into_iter().map(Into::into).collect();
        self
    }

    pub fn build(self) -> Snapshot {
        let mut rng = rand::rngs::OsRng;

        let count = {
            if self.count == 0 {
                rng.gen_range(1usize, 1_000usize)
            } else {
                self.count
            }
        };

        Snapshot {
            tag: self.tag.clone(),
            content: std::iter::from_fn(|| {
                Some(VoterHIR {
                    voting_key: TestGen::identifier().into(),
                    voting_group: self.groups[rng.gen_range(0, self.groups.len())].to_string(),
                    voting_power: rng.gen_range(1u64, 1_000u64).into(),
                })
            })
            .take(count)
            .collect(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct VotingPower {
    pub voting_power: u64,
    pub voting_group: String,
}

impl From<VoterHIR> for VotingPower {
    fn from(voter_hir: VoterHIR) -> Self {
        Self {
            voting_power: voter_hir.voting_power.into(),
            voting_group: voter_hir.voting_group,
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
                    .map(|x| x.voting_group.clone())
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
            let mut voting_power: u64 = entry.voting_power.into();
            voting_power += rng.gen_range(1u64, 1_000u64);
            entry.voting_power = voting_power.into();
        }
        self
    }

    pub fn build(self) -> Snapshot {
        self.snapshot
    }
}
