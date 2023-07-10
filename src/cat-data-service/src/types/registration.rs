use super::SerdeType;
use event_db::types::registration::{Delegation, Delegator, Voter, VoterGroupId, VoterInfo};
use serde::{
    de::Deserializer,
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

impl Serialize for SerdeType<&VoterGroupId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0 .0.serialize(serializer)
    }
}

impl Serialize for SerdeType<VoterGroupId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerdeType<VoterGroupId> {
    fn deserialize<D>(deserializer: D) -> Result<SerdeType<VoterGroupId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VoterGroupId(String::deserialize(deserializer)?).into())
    }
}

impl Serialize for SerdeType<&VoterInfo> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("VoterInfo", 6)?;
        serializer.serialize_field("voting_power", &self.voting_power)?;
        serializer.serialize_field("voting_group", &SerdeType(&self.voting_group))?;
        serializer.serialize_field("delegations_power", &self.delegations_power)?;
        serializer.serialize_field("delegations_count", &self.delegations_count)?;
        serializer.serialize_field("voting_power_saturation", &self.voting_power_saturation)?;
        if let Some(delegator_addresses) = &self.delegator_addresses {
            serializer.serialize_field("delegator_addresses", delegator_addresses)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<VoterInfo> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Voter> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("VoterInfo", 4)?;
        serializer.serialize_field("voter_info", &SerdeType(&self.voter_info))?;
        serializer.serialize_field("as_at", &self.as_at.to_rfc3339())?;
        serializer.serialize_field("last_updated", &self.last_updated.to_rfc3339())?;
        serializer.serialize_field("final", &self.is_final)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<Voter> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Delegation> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("Delegation", 4)?;
        serializer.serialize_field("voting_key", &self.voting_key)?;
        serializer.serialize_field("group", &SerdeType(&self.group))?;
        serializer.serialize_field("weight", &self.weight)?;
        serializer.serialize_field("value", &self.value)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<Delegation> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Delegator> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("Delegator", 8)?;
        serializer.serialize_field(
            "delegations",
            &self.delegations.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        serializer.serialize_field("reward_address", self.reward_address.reward_address())?;
        serializer.serialize_field("reward_payable", &self.reward_address.reward_payable())?;
        serializer.serialize_field("raw_power", &self.raw_power)?;
        serializer.serialize_field("total_power", &self.total_power)?;
        serializer.serialize_field("as_at", &self.as_at.to_rfc3339())?;
        serializer.serialize_field("last_updated", &self.last_updated.to_rfc3339())?;
        serializer.serialize_field("final", &self.is_final)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<Delegator> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use event_db::types::registration::RewardAddress;
    use serde_json::json;

    #[test]
    fn voter_group_id_json_test() {
        let voter_group_id = SerdeType(VoterGroupId("rep".to_string()));

        let json = serde_json::to_value(&voter_group_id).unwrap();
        assert_eq!(json, json!("rep"));

        let expected: SerdeType<VoterGroupId> = serde_json::from_value(json).unwrap();
        assert_eq!(expected, voter_group_id);
    }

    #[test]
    fn voter_info_json_test() {
        let voter_info = SerdeType(VoterInfo {
            voting_power: 100,
            voting_group: VoterGroupId("group".to_string()),
            delegations_power: 100,
            delegations_count: 1,
            voting_power_saturation: 1.0,
            delegator_addresses: Some(vec!["stake_public_key_1".to_string()]),
        });

        let json = serde_json::to_value(&voter_info).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voting_power": 100,
                    "voting_group": "group",
                    "delegations_power": 100,
                    "delegations_count": 1,
                    "voting_power_saturation": 1.0,
                    "delegator_addresses": ["stake_public_key_1"]
                }
            )
        );

        let voter_info = SerdeType(VoterInfo {
            voting_power: 100,
            voting_group: VoterGroupId("group".to_string()),
            delegations_power: 100,
            delegations_count: 1,
            voting_power_saturation: 1.0,
            delegator_addresses: None,
        });

        let json = serde_json::to_value(&voter_info).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voting_power": 100,
                    "voting_group": "group",
                    "delegations_power": 100,
                    "delegations_count": 1,
                    "voting_power_saturation": 1.0,
                }
            )
        );
    }

    #[test]
    fn voter_json_test() {
        let voter = SerdeType(Voter {
            voter_info: VoterInfo {
                voting_power: 100,
                voting_group: VoterGroupId("group".to_string()),
                delegations_power: 100,
                delegations_count: 1,
                voting_power_saturation: 1.0,
                delegator_addresses: None,
            },
            as_at: DateTime::from_utc(NaiveDateTime::default(), Utc),
            last_updated: DateTime::from_utc(NaiveDateTime::default(), Utc),
            is_final: true,
        });

        let json = serde_json::to_value(&voter).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voter_info": {
                        "voting_power": 100,
                        "voting_group": "group",
                        "delegations_power": 100,
                        "delegations_count": 1,
                        "voting_power_saturation": 1.0,
                    },
                    "as_at": "1970-01-01T00:00:00+00:00",
                    "last_updated": "1970-01-01T00:00:00+00:00",
                    "final": true
                }
            )
        )
    }

    #[test]
    fn delegation_json_test() {
        let delegation = SerdeType(Delegation {
            voting_key: "stake_public_key_1".to_string(),
            group: VoterGroupId("group".to_string()),
            weight: 100,
            value: 100,
        });

        let json = serde_json::to_value(&delegation).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "voting_key": "stake_public_key_1",
                    "group": "group",
                    "weight": 100,
                    "value": 100,
                }
            )
        );
    }

    #[test]
    fn delegator_json_test() {
        let delegator = SerdeType(Delegator {
            delegations: vec![],
            reward_address: RewardAddress::new("stake_public_key_1".to_string()),
            raw_power: 100,
            total_power: 100,
            as_at: DateTime::from_utc(NaiveDateTime::default(), Utc),
            last_updated: DateTime::from_utc(NaiveDateTime::default(), Utc),
            is_final: true,
        });

        let json = serde_json::to_value(&delegator).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "delegations": [],
                    "reward_address": "stake_public_key_1",
                    "reward_payable": false,
                    "raw_power": 100,
                    "total_power": 100,
                    "as_at": "1970-01-01T00:00:00+00:00",
                    "last_updated": "1970-01-01T00:00:00+00:00",
                    "final": true
                }
            )
        );
    }
}
