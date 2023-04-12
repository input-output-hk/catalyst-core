use serde::Serialize;

use crate::error::Error;

use super::VoterGroupId;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ObjectiveTypes {
    #[serde(rename = "catalyst-simple")]
    CatalystSimple,
    #[serde(rename = "catalyst-native")]
    CatalystNative,
    #[serde(rename = "catalyst-community-choice")]
    CatalystCommunityChoice,
    #[serde(rename = "sve-decision")]
    SveDecision,
}

impl TryFrom<String> for ObjectiveTypes {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if &value == "catalyst-simple" {
            Ok(Self::CatalystSimple)
        } else if &value == "catalyst-native" {
            Ok(Self::CatalystNative)
        } else if &value == "catalyst-community-choice" {
            Ok(Self::CatalystCommunityChoice)
        } else if &value == "sve-decision" {
            Ok(Self::SveDecision)
        } else {
            Err(Error::Unknown(format!(
                "Could be only one of the following options: [catalyst-simple, catalyst-native, catalyst-community-choice, sve-decision], provided: {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveType {
    pub id: ObjectiveTypes,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveSummary {
    pub id: i32,
    #[serde(rename = "type")]
    pub objective_type: ObjectiveType,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum ObjectiveProposalType {
    #[serde(rename = "simple")]
    Simple,
    #[serde(rename = "native")]
    Native,
    #[serde(rename = "community-choice")]
    CommunityChoice,
}

impl TryFrom<String> for ObjectiveProposalType {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if &value == "simple" {
            Ok(Self::Simple)
        } else if &value == "native" {
            Ok(Self::Native)
        } else if &value == "community-choice" {
            Ok(Self::CommunityChoice)
        } else {
            Err(Error::Unknown(format!(
                "Could be only one of the following options: [simple, native, community-choice], provided: {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum RewardCurrency {
    #[serde(rename = "USD_ADA")]
    UsdAda,
    #[serde(rename = "ADA")]
    Ada,
    #[serde(rename = "CLAP")]
    Clap,
    #[serde(rename = "COTI")]
    Coti,
}

impl TryFrom<String> for RewardCurrency {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if &value == "USD_ADA" {
            Ok(Self::UsdAda)
        } else if &value == "ADA" {
            Ok(Self::Ada)
        } else if &value == "CLAP" {
            Ok(Self::Clap)
        } else if &value == "COTI" {
            Ok(Self::Coti)
        } else {
            Err(Error::Unknown(format!(
                "Could be only one of the following options: [USD_ADA, ADA, CLAP, COTI], provided: {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RewardDefintion {
    pub currency: RewardCurrency,
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum BallotType {
    #[serde(rename = "public")]
    Public,
    #[serde(rename = "private")]
    Private,
    #[serde(rename = "cast-private")]
    CastPrivate,
}

impl TryFrom<String> for BallotType {
    type Error = Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if &value == "public" {
            Ok(Self::Public)
        } else if &value == "private" {
            Ok(Self::Private)
        } else if &value == "cast-private" {
            Ok(Self::CastPrivate)
        } else {
            Err(Error::Unknown(format!(
                "Could be only one of the following options: [public, private, cast-private], provided: {}",
                value
            )))
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GroupBallotType {
    pub group: VoterGroupId,
    pub ballot: BallotType,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveSupplementalData {
    pub sponsor: String,
    pub video: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveDetails {
    pub description: String,
    #[serde(rename = "type")]
    pub objective_proposal_type: ObjectiveProposalType,
    pub reward: RewardDefintion,
    pub choices: Vec<String>,
    pub ballot: GroupBallotType,
    pub url: String,
    pub supplemental: ObjectiveSupplementalData,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Objective {
    #[serde(flatten)]
    summary: ObjectiveSummary,
    #[serde(flatten)]
    details: ObjectiveDetails,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn objective_type_json_test() {
        let objective_type = ObjectiveType {
            id: ObjectiveTypes::CatalystNative,
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
            id: 1,
            objective_type: ObjectiveType {
                id: ObjectiveTypes::CatalystNative,
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
            currency: RewardCurrency::Ada,
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
            group: VoterGroupId::Rep,
            ballot: BallotType::Public,
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
            objective_proposal_type: ObjectiveProposalType::Native,
            reward: RewardDefintion {
                currency: RewardCurrency::Ada,
                value: 100,
            },
            choices: vec!["Abstain".to_string(), "Yes".to_string(), "No".to_string()],
            ballot: GroupBallotType {
                group: VoterGroupId::Rep,
                ballot: BallotType::Public,
            },
            url: "objective url 1".to_string(),
            supplemental: ObjectiveSupplementalData {
                sponsor: "sponsor 1".to_string(),
                video: "video url 1".to_string(),
            },
        };

        let json = serde_json::to_value(&objective_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "description": "objective 1",
                    "type": "native",
                    "reward": {
                        "currency": "ADA",
                        "value": 100,
                    },
                    "choices": ["Abstain", "Yes", "No"],
                    "ballot": {
                        "group": "rep",
                        "ballot": "public",
                    },
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
