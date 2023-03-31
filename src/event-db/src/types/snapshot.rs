use crate::types::utils::serialize_datetime_as_rfc3339;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotVersion(pub i32);

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct VoterInfo {
    pub voting_power: i64,
    pub voting_group: String,
    pub delegations_power: i64,
    pub delegations_count: i64,
    pub voting_power_saturation: f64,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Voter {
    pub voter_info: VoterInfo,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub as_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "final")]
    pub is_final: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Delegation {
    pub voting_key: String,
    pub group: String,
    pub weight: i32,
    pub value: i64,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Delegator {
    pub delegations: Vec<Delegation>,
    pub raw_power: i64,
    pub total_power: i64,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub as_at: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub last_updated: DateTime<Utc>,
    #[serde(rename = "final")]
    pub is_final: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn snapshot_version_json_test() {
        let snapshot_versions = vec![
            SnapshotVersion(10),
            SnapshotVersion(11),
            SnapshotVersion(12),
        ];
        let json = serde_json::to_value(&snapshot_versions).unwrap();
        assert_eq!(json, json!([10, 11, 12]));

        let decoded: Vec<SnapshotVersion> = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, snapshot_versions);
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
            as_at: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            last_updated: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
        };
        let json = serde_json::to_value(&voter).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voter_info": {
                            "voting_power": 100,
                            "voting_group": "rep",
                            "delegations_power": 100,
                            "delegations_count": 1,
                            "voting_power_saturation": 0.4
                        },
                    "as_at": "1970-01-01T00:00:00+00:00",
                    "last_updated": "1970-01-01T00:00:00+00:00",
                    "final": true
                }
            )
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
            as_at: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            last_updated: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
        };
        let json = serde_json::to_value(&delegator).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "delegations": [{"voting_key": "voter","group": "rep","weight": 5,"value": 100}],
                    "raw_power": 100,
                    "total_power": 1000,
                    "as_at": "1970-01-01T00:00:00+00:00",
                    "last_updated": "1970-01-01T00:00:00+00:00",
                    "final": true
                }
            )
        );
    }
}
