use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct VoteOptions(pub HashMap<String, u8>);

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct Proposal {
    #[serde(alias = "internalId")]
    pub internal_id: i32,
    #[serde(alias = "proposalId")]
    pub proposal_id: String,
    #[serde(alias = "proposalTitle")]
    pub proposal_title: String,
    #[serde(alias = "proposalFunds")]
    pub proposal_funds: i64,
    #[serde(alias = "proposalUrl")]
    pub proposal_url: String,
    #[serde(alias = "proposalImpactScore")]
    pub proposal_impact_score: i64,
    #[serde(alias = "chainProposalId")]
    pub chain_proposal_id: Vec<u8>,
    #[serde(alias = "chainVoteOptions")]
    pub chain_vote_options: VoteOptions,
    #[serde(alias = "challengeId")]
    pub challenge_id: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_json_test() {
        let json = serde_json::json!({
            "internal_id": 1,
            "proposal_id": "1",
            "proposal_title": "test",
            "proposal_funds": 1,
            "proposal_url": "test",
            "proposal_impact_score": 1,
            "chain_proposal_id": [1],
            "chain_vote_options": {
                "1": 1
            },
            "challenge_id": 1,
            "not_proposal_data": "some",
        });

        let proposal: Proposal = serde_json::from_value(json).unwrap();

        assert_eq!(
            proposal,
            Proposal{
                internal_id: 1,
                proposal_id: "1".to_string(),
                proposal_title: "test".to_string(),
                proposal_funds: 1,
                proposal_url: "test".to_string(),
                proposal_impact_score: 1,
                chain_proposal_id: vec![1],
                chain_vote_options: VoteOptions(HashMap::from([("1".to_string(), 1)])),
                challenge_id: 1
            }
        );
    }
}
