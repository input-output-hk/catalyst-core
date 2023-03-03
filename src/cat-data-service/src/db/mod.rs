use self::snapshot::{Delegation, Delegator, SnapshotDb, SnapshotVersions, Voter, VoterInfo};

pub mod snapshot;

pub trait DB: SnapshotDb {}

#[derive(Clone)]
pub struct MockedDB {
    pub voter: Voter,
    pub delegator: Delegator,
    pub snapshot_versions: SnapshotVersions,
}

impl Default for MockedDB {
    fn default() -> Self {
        Self {
            snapshot_versions: SnapshotVersions(vec!["latest".to_string(), "fund 10".to_string()]),
            voter: Voter {
                voter_info: VoterInfo {
                    voting_power: 100,
                    voting_group: "rep".to_string(),
                    delegations_power: 100,
                    delegations_count: 1,
                    voting_power_saturation: 0.4,
                },
                as_at: "today".to_string(),
                last_updated: "today".to_string(),
                r#final: true,
            },
            delegator: Delegator {
                delegations: vec![Delegation {
                    voting_key: "voter".to_string(),
                    group: "rep".to_string(),
                    weight: 5,
                    value: 100,
                }],
                raw_power: 100,
                total_power: 1000,
                as_at: "today".to_string(),
                last_updated: "today".to_string(),
                r#final: true,
            },
        }
    }
}

impl SnapshotDb for MockedDB {
    fn get_snapshot_versions(&self) -> SnapshotVersions {
        self.snapshot_versions.clone()
    }
    fn get_voter(&self, _event: String, _voting_key: String) -> Voter {
        self.voter.clone()
    }
    fn get_delegator(&self, _event: String, _stake_public_key: String) -> Delegator {
        self.delegator.clone()
    }
}

impl DB for MockedDB {}
