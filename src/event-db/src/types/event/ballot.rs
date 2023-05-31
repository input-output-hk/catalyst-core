use crate::types::registration::VoterGroupId;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ObjectiveChoices(pub Vec<String>);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BallotType(pub String);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VotePlan {
    pub chain_proposal_index: i64,
    pub group: VoterGroupId,
    pub ballot_type: BallotType,
    pub chain_voteplan_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GroupVotePlans(pub Vec<VotePlan>);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Ballot {
    pub choices: ObjectiveChoices,
    pub voteplans: GroupVotePlans,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ballot_json_test() {
        let ballot = Ballot {
            choices: ObjectiveChoices(vec![
                "Abstain".to_string(),
                "Yes".to_string(),
                "No".to_string(),
            ]),
            voteplans: GroupVotePlans(vec![VotePlan {
                chain_proposal_index: 1,
                group: VoterGroupId("rep".to_string()),
                ballot_type: BallotType("public".to_string()),
                chain_voteplan_id: "chain_voteplan_id 1".to_string(),
                encryption_key: Some("encryption_key 1".to_string()),
            }]),
        };

        let json = serde_json::to_value(&ballot).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "choices": [
                        "Abstain",
                        "Yes",
                        "No"
                    ],
                    "voteplans": [
                        {
                            "chain_proposal_index": 1,
                            "group": "rep",
                            "ballot_type": "public",
                            "chain_voteplan_id": "chain_voteplan_id 1",
                            "encryption_key": "encryption_key 1"
                        }
                    ]
                }
            )
        )
    }
}
