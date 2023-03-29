use crate::types::utils::serialize_systemtime_as_rfc3339;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotVersion {
    Latest,
    Number(i32),
    Name(String),
}

impl Serialize for SnapshotVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Latest => "latest".serialize(serializer),
            Self::Number(number) => number.to_string().serialize(serializer),
            Self::Name(name) => name.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SnapshotVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SnapshotVersionVisitor;
        impl<'de> serde::de::Visitor<'de> for SnapshotVersionVisitor {
            type Value = SnapshotVersion;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str(
                    r#"Expect one of the following options: "latest", "25", "Fund 10" etc."#,
                )
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v == "latest" {
                    Ok(Self::Value::Latest)
                } else {
                    match v.parse::<i32>() {
                        Ok(number) => Ok(Self::Value::Number(number)),
                        _ => Ok(Self::Value::Name(v.to_string())),
                    }
                }
            }
        }
        deserializer.deserialize_any(SnapshotVersionVisitor)
    }
}

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
    use serde_json::json;

    use super::*;

    #[test]
    fn snapshot_version_json_test() {
        let snapshot_versions = vec![
            SnapshotVersion::Latest,
            SnapshotVersion::Number(10),
            SnapshotVersion::Name("Fund 10".to_string()),
        ];
        let json = serde_json::to_value(&snapshot_versions).unwrap();
        assert_eq!(json, json!(["latest", "10", "Fund 10"]));

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
            as_at: SystemTime::UNIX_EPOCH,
            last_updated: SystemTime::UNIX_EPOCH,
            r#final: true,
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
                    "as_at": "1970-01-01T00:00:00Z",
                    "last_updated": "1970-01-01T00:00:00Z",
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
            as_at: SystemTime::UNIX_EPOCH,
            last_updated: SystemTime::UNIX_EPOCH,
            r#final: true,
        };
        let json = serde_json::to_value(&delegator).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "delegations": [{"voting_key": "voter","group": "rep","weight": 5,"value": 100}],
                    "raw_power": 100,
                    "total_power": 1000,
                    "as_at": "1970-01-01T00:00:00Z",
                    "last_updated": "1970-01-01T00:00:00Z",
                    "final": true
                }
            )
        );
    }
}
