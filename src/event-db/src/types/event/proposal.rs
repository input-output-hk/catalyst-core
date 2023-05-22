use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProposalId(pub i32);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposerDetails {
    pub name: String,
    pub email: String,
    pub url: String,
    pub payment_key: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VotePlan {
    pub group: String,
    pub chain_voteplan_id: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposalSupplementalDetails(pub Value);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProposalDetails {
    pub funds: i64,
    pub url: String,
    pub files: String,
    pub proposer: Vec<ProposerDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supplemental: Option<ProposalSupplementalDetails>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ProposalSummary {
    pub id: i32,
    pub title: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Proposal {
    #[serde(flatten)]
    pub proposal_summary: ProposalSummary,
    #[serde(flatten)]
    pub proposal_details: ProposalDetails,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn proposer_details_json_test() {
        let proposer_details = ProposerDetails {
            name: "proposer name".to_string(),
            email: "proposer email".to_string(),
            url: "proposer url".to_string(),
            payment_key: "proposer payment key".to_string(),
        };

        let json = serde_json::to_value(&proposer_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "name": "proposer name",
                    "email": "proposer email",
                    "url": "proposer url",
                    "payment_key": "proposer payment key",
                }
            )
        );
    }

    #[test]
    fn proposal_supplemental_details_json_test() {
        let proposal_supplemental_details = ProposalSupplementalDetails(json!(
                {
                    "solution": "solution",
                    "brief": "brief",
                    "importance": "importance",
                    "metrics": "metrics",
                }
        ));

        let json = serde_json::to_value(&proposal_supplemental_details).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "solution": "solution",
                    "brief": "brief",
                    "importance": "importance",
                    "metrics": "metrics",
                }
            )
        );
    }
}
