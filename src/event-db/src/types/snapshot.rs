use serde::Serialize;
use std::time::SystemTime;

#[derive(Serialize, Clone, Default)]
pub struct VoterInfo {
    pub voting_power: i64,
    pub voting_group: String,
    pub delegations_power: u64,
    pub delegations_count: u64,
    pub voting_power_saturation: f64,
}

#[derive(Serialize, Clone)]
pub struct Voter {
    pub voter_info: VoterInfo,
    pub as_at: SystemTime,
    pub last_updated: SystemTime,
    pub r#final: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct Delegation {
    pub voting_key: String,
    pub group: String,
    pub weight: u64,
    pub value: u64,
}

#[derive(Serialize, Clone, Default)]
pub struct Delegator {
    pub delegations: Vec<Delegation>,
    pub raw_power: u64,
    pub total_power: u64,
    pub as_at: String,
    pub last_updated: String,
    pub r#final: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_versions_json_test() {
        let snapshot_versions = vec!["latest".to_string(), "fund 10".to_string()];
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
            as_at: SystemTime::UNIX_EPOCH,
            last_updated: SystemTime::UNIX_EPOCH,
            r#final: true,
        };
        let json = serde_json::to_string(&voter).unwrap();
        assert_eq!(
            json,
            r#"{"voter_info":{"voting_power":100,"voting_group":"rep","delegations_power":100,"delegations_count":1,"voting_power_saturation":0.4},"as_at":{"secs_since_epoch":0,"nanos_since_epoch":0},"last_updated":{"secs_since_epoch":0,"nanos_since_epoch":0},"final":true}"#
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
