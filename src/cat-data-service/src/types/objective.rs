use super::SerdeType;
use event_db::types::event::objective::{
    Objective, ObjectiveDetails, ObjectiveId, ObjectiveSummary, ObjectiveType, RewardDefintion,
    VoterGroup,
};
use serde::{
    de::Deserializer,
    ser::{SerializeStruct, Serializer},
    Deserialize, Serialize,
};

impl Serialize for SerdeType<&ObjectiveId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0 .0.serialize(serializer)
    }
}

impl Serialize for SerdeType<ObjectiveId> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SerdeType<ObjectiveId> {
    fn deserialize<D>(deserializer: D) -> Result<SerdeType<ObjectiveId>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(ObjectiveId(i32::deserialize(deserializer)?).into())
    }
}

impl Serialize for SerdeType<&ObjectiveType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ObjectiveType", 2)?;
        serializer.serialize_field("id", &self.id)?;
        serializer.serialize_field("description", &self.description)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ObjectiveType> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ObjectiveSummary> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ObjectiveSummary", 4)?;
        serializer.serialize_field("id", &SerdeType(&self.id))?;
        serializer.serialize_field("type", &SerdeType(&self.objective_type))?;
        serializer.serialize_field("title", &self.title)?;
        serializer.serialize_field("description", &self.description)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<ObjectiveSummary> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&RewardDefintion> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("RewardDefintion", 2)?;
        serializer.serialize_field("currency", &self.currency)?;
        serializer.serialize_field("value", &self.value)?;
        serializer.end()
    }
}

impl Serialize for SerdeType<RewardDefintion> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&VoterGroup> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("VoterGroup", 2)?;
        if let Some(group) = &self.group {
            serializer.serialize_field("group", &SerdeType(group))?;
        }
        if let Some(voting_token) = &self.voting_token {
            serializer.serialize_field("voting_token", voting_token)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<VoterGroup> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&ObjectiveDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serializer = serializer.serialize_struct("ObjectiveDetails", 3)?;
        serializer.serialize_field(
            "groups",
            &self.groups.iter().map(SerdeType).collect::<Vec<_>>(),
        )?;
        if let Some(reward) = &self.reward {
            serializer.serialize_field("reward", &SerdeType(reward))?;
        }
        if let Some(supplemental) = &self.supplemental {
            serializer.serialize_field("supplemental", &supplemental)?;
        }
        serializer.end()
    }
}

impl Serialize for SerdeType<ObjectiveDetails> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerdeType(&self.0).serialize(serializer)
    }
}

impl Serialize for SerdeType<&Objective> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        pub struct ObjectiveImpl<'a> {
            #[serde(flatten)]
            summary: SerdeType<&'a ObjectiveSummary>,
            #[serde(flatten)]
            details: SerdeType<&'a ObjectiveDetails>,
        }

        let val = ObjectiveImpl {
            summary: SerdeType(&self.summary),
            details: SerdeType(&self.details),
        };
        val.serialize(serializer)
    }
}

impl Serialize for SerdeType<Objective> {
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
    use event_db::types::registration::VoterGroupId;
    use serde_json::json;

    #[test]
    fn objective_id_json_test() {
        let objective_id = SerdeType(ObjectiveId(1));

        let json = serde_json::to_value(&objective_id).unwrap();
        assert_eq!(json, json!(1));

        let expected: SerdeType<ObjectiveId> = serde_json::from_value(json).unwrap();
        assert_eq!(expected, objective_id);
    }

    #[test]
    fn objective_type_json_test() {
        let objective_type = SerdeType(ObjectiveType {
            id: "catalyst-native".to_string(),
            description: "catalyst native type".to_string(),
        });

        let json = serde_json::to_value(&objective_type).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": "catalyst-native",
                    "description": "catalyst native type",
                }
            )
        );
    }

    #[test]
    fn objective_summary_json_test() {
        let objective_summary = SerdeType(ObjectiveSummary {
            id: ObjectiveId(1),

            objective_type: ObjectiveType {
                id: "catalyst-native".to_string(),
                description: "catalyst native type".to_string(),
            },
            title: "objective 1".to_string(),
            description: "description 1".to_string(),
        });

        let json = serde_json::to_value(&objective_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "type": {
                        "id": "catalyst-native",
                        "description": "catalyst native type",
                    },
                    "title": "objective 1",
                    "description": "description 1",
                }
            )
        );
    }

    #[test]
    fn reward_definition_json_test() {
        let reward_definition = SerdeType(RewardDefintion {
            currency: "ADA".to_string(),
            value: 100,
        });

        let json = serde_json::to_value(&reward_definition).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "currency": "ADA",
                    "value": 100
                }
            )
        )
    }

    #[test]
    fn voter_group_json_test() {
        let voter_group = SerdeType(VoterGroup {
            group: Some(VoterGroupId("group".to_string())),
            voting_token: Some("token".to_string()),
        });

        let json = serde_json::to_value(&voter_group).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "group": "group",
                    "voting_token": "token"
                }
            )
        );
    }

    #[test]
    fn objective_details_json_test() {
        let objective_details = SerdeType(ObjectiveDetails {
            groups: vec![VoterGroup {
                group: Some(VoterGroupId("group".to_string())),
                voting_token: Some("token".to_string()),
            }],
            reward: Some(RewardDefintion {
                currency: "ADA".to_string(),
                value: 100,
            }),
            supplemental: None,
        });

        let json = serde_json::to_value(&objective_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "groups": [
                        {
                            "group": "group",
                            "voting_token": "token"
                        }
                    ],
                    "reward": {
                        "currency": "ADA",
                        "value": 100
                    }
                }
            )
        )
    }

    #[test]
    fn objective_json_test() {
        let objective = SerdeType(Objective {
            summary: ObjectiveSummary {
                id: ObjectiveId(1),

                objective_type: ObjectiveType {
                    id: "catalyst-native".to_string(),
                    description: "catalyst native type".to_string(),
                },
                title: "objective 1".to_string(),
                description: "description 1".to_string(),
            },
            details: ObjectiveDetails {
                groups: vec![VoterGroup {
                    group: Some(VoterGroupId("group".to_string())),
                    voting_token: Some("token".to_string()),
                }],
                reward: Some(RewardDefintion {
                    currency: "ADA".to_string(),
                    value: 100,
                }),
                supplemental: None,
            },
        });

        let json = serde_json::to_value(&objective).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "type": {
                        "id": "catalyst-native",
                        "description": "catalyst native type",
                    },
                    "title": "objective 1",
                    "description": "description 1",
                    "groups": [
                        {
                            "group": "group",
                            "voting_token": "token"
                        }
                    ],
                    "reward": {
                        "currency": "ADA",
                        "value": 100
                    }
                }
            )
        )
    }
}
