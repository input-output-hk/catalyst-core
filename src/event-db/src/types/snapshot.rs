use crate::types::utils::serialize_systemtime_as_rfc3339;
use serde::Serialize;
use std::time::SystemTime;

#[derive(Serialize, Clone, Default)]
pub struct VoterInfo {
    pub voting_power: i64,
    pub voting_group: String,
    pub delegations_power: i64,
    pub delegations_count: i64,
    pub voting_power_saturation: f64,
}

#[derive(Serialize, Clone)]
pub struct Voter {
    pub voter_info: VoterInfo,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    pub as_at: SystemTime,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    pub last_updated: SystemTime,
    pub r#final: bool,
}

#[derive(Serialize, Clone, Default)]
pub struct Delegation {
    pub voting_key: String,
    pub group: String,
    pub weight: i32,
    pub value: i64,
}

#[derive(Serialize, Clone)]
pub struct Delegator {
    pub delegations: Vec<Delegation>,
    pub raw_power: i64,
    pub total_power: i64,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    pub as_at: SystemTime,
    #[serde(serialize_with = "serialize_systemtime_as_rfc3339")]
    pub last_updated: SystemTime,
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
            r#"{"voter_info":{"voting_power":100,"voting_group":"rep","delegations_power":100,"delegations_count":1,"voting_power_saturation":0.4},"as_at":"1970-01-01T00:00:00Z","last_updated":"1970-01-01T00:00:00Z","final":true}"#
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
            as_at: SystemTime::UNIX_EPOCH,
            last_updated: SystemTime::UNIX_EPOCH,
            r#final: true,
        };
        let json = serde_json::to_string(&delegator).unwrap();
        assert_eq!(
            json,
            r#"{"delegations":[{"voting_key":"voter","group":"rep","weight":5,"value":100}],"raw_power":100,"total_power":1000,"as_at":"1970-01-01T00:00:00Z","last_updated":"1970-01-01T00:00:00Z","final":true}"#
        );
    }
}
