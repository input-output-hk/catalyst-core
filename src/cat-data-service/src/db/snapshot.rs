use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SnapshotVersions(pub Vec<String>);

#[derive(Serialize, Clone)]
pub struct VoterInfo {
    pub voting_power: u64,
    pub voting_group: String,
    pub delegations_power: u64,
    pub delegations_count: u64,
    pub voting_power_saturation: f64,
}

#[derive(Serialize, Clone)]
pub struct Voter {
    pub voter_info: VoterInfo,
    pub as_at: String,
    pub last_updated: String,
    pub r#final: bool,
}

#[derive(Serialize, Clone)]
pub struct Delegation {
    pub voting_key: String,
    pub group: String,
    pub weight: u64,
    pub value: u64,
}

#[derive(Serialize, Clone)]
pub struct Delegator {
    pub delegations: Vec<Delegation>,
    pub raw_power: u64,
    pub total_power: u64,
    pub as_at: String,
    pub last_updated: String,
    pub r#final: bool,
}

pub trait SnapshotDb {
    fn get_snapshot_versions(&self) -> SnapshotVersions;
    fn get_voter(&self, event: String, voting_key: String) -> Voter;
    fn get_delegator(&self, event: String, stake_public_key: String) -> Delegator;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_versions_json_test() {
        let snapshot_versions = SnapshotVersions(vec!["latest".to_string(), "fund 10".to_string()]);
        let json = serde_json::to_string(&snapshot_versions).unwrap();
        assert_eq!(json, r#"["latest","fund 10"]"#);
    }

    #[test]
    fn voter_json_test() {
        let voter = Voter {
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
        };
        let json = serde_json::to_string(&voter).unwrap();
        assert_eq!(
            json,
            r#"{"voter_info":{"voting_power":100,"voting_group":"rep","delegations_power":100,"delegations_count":1,"voting_power_saturation":0.4},"as_at":"today","last_updated":"today","final":true}"#
        );
    }

    #[test]
    fn delegator_json_test() {
        let delegator = Delegator {
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
        };
        let json = serde_json::to_string(&delegator).unwrap();
        assert_eq!(
            json,
            r#"{"delegations":[{"voting_key":"voter","group":"rep","weight":5,"value":100}],"raw_power":100,"total_power":1000,"as_at":"today","last_updated":"today","final":true}"#
        );
    }
}
