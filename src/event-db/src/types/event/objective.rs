use crate::types::registration::VoterGroupId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectiveId(pub i32);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveType {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveSummary {
    pub id: ObjectiveId,
    #[serde(rename = "type")]
    pub objective_type: ObjectiveType,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RewardDefintion {
    pub currency: String,
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct VoterGroup {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<VoterGroupId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voting_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveSupplementalData(pub Value);
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveDetails {
    pub groups: Vec<VoterGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<RewardDefintion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplemental: Option<ObjectiveSupplementalData>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Objective {
    #[serde(flatten)]
    pub summary: ObjectiveSummary,
    #[serde(flatten)]
    pub details: ObjectiveDetails,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn objective_type_json_test() {
        let objective_type = ObjectiveType {
            id: "catalyst-native".to_string(),
            description: "catalyst native type".to_string(),
        };

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
        let objective_summary = ObjectiveSummary {
            id: ObjectiveId(1),
            objective_type: ObjectiveType {
                id: "catalyst-native".to_string(),
                description: "catalyst native type".to_string(),
            },
            title: "objective 1".to_string(),
            description: "description 1".to_string(),
        };

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
        let reward_definition = RewardDefintion {
            currency: "ADA".to_string(),
            value: 100,
        };

        let json = serde_json::to_value(&reward_definition).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "currency": "ADA",
                    "value": 100,
                }
            )
        );
    }

    #[test]
    fn voter_group_json_test() {
        let voter_group = VoterGroup {
            group: Some(VoterGroupId("rep".to_string())),
            voting_token: Some("voting token 1".to_string()),
        };

        let json = serde_json::to_value(&voter_group).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "group": "rep",
                    "voting_token": "voting token 1",
                }
            )
        );
    }

    #[test]
    fn objective_details_json_test() {
        let objective_details = ObjectiveDetails {
            groups: vec![VoterGroup {
                group: Some(VoterGroupId("rep".to_string())),
                voting_token: Some("voting token 1".to_string()),
            }],
            reward: Some(RewardDefintion {
                currency: "ADA".to_string(),
                value: 100,
            }),
            supplemental: Some(ObjectiveSupplementalData(json!(
                {
                    "url": "objective url 1",
                    "sponsor": "sponsor 1",
                    "video": "video url 1",
                }
            ))),
        };

        let json = serde_json::to_value(&objective_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "groups": [
                        {
                            "group": "rep",
                            "voting_token": "voting token 1",
                        }
                    ],
                    "reward": {
                        "currency": "ADA",
                        "value": 100,
                    },
                    "supplemental": {
                        "url": "objective url 1",
                        "sponsor": "sponsor 1",
                        "video": "video url 1",
                    },
                }
            )
        );
    }
}
