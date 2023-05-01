use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RewardDefintion {
    pub currency: String,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GroupBallotType {
    pub group: String,
    pub ballot: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveSupplementalData {
    pub sponsor: String,
    pub video: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveDetails {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reward: Option<RewardDefintion>,
    pub choices: Vec<String>,
    pub ballot: Vec<GroupBallotType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
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
    fn group_ballot_type_json_test() {
        let group_ballot_type = GroupBallotType {
            group: "rep".to_string(),
            ballot: "public".to_string(),
        };

        let json = serde_json::to_value(&group_ballot_type).unwrap();
        assert_eq!(
            json,
            json!({
                "group": "rep",
                "ballot": "public",
            })
        );
    }

    #[test]
    fn objective_supplemental_data_json_test() {
        let objective_supplemental_data = ObjectiveSupplementalData {
            sponsor: "sponsor 1".to_string(),
            video: "video url 1".to_string(),
        };

        let json = serde_json::to_value(&objective_supplemental_data).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "sponsor": "sponsor 1",
                    "video": "video url 1",
                }
            )
        );
    }

    #[test]
    fn objective_details_json_test() {
        let objective_details = ObjectiveDetails {
            description: "objective 1".to_string(),
            reward: Some(RewardDefintion {
                currency: "ADA".to_string(),
                value: 100,
            }),
            choices: vec!["Abstain".to_string(), "Yes".to_string(), "No".to_string()],
            ballot: vec![GroupBallotType {
                group: "rep".to_string(),
                ballot: "public".to_string(),
            }],
            url: Some("objective url 1".to_string()),
            supplemental: Some(ObjectiveSupplementalData {
                sponsor: "sponsor 1".to_string(),
                video: "video url 1".to_string(),
            }),
        };

        let json = serde_json::to_value(&objective_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "description": "objective 1",
                    "reward": {
                        "currency": "ADA",
                        "value": 100,
                    },
                    "choices": ["Abstain", "Yes", "No"],
                    "ballot": [{
                        "group": "rep",
                        "ballot": "public",
                    }],
                    "url": "objective url 1",
                    "supplemental": {
                        "sponsor": "sponsor 1",
                        "video": "video url 1",
                    },
                }
            )
        );
    }
}
