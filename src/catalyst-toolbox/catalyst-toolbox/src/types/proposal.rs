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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct ProposalVotePlanCommon {
    #[serde(alias = "chainVoteplanId")]
    pub chain_voteplan_id: String,
    #[serde(alias = "chainProposalIndex")]
    pub chain_proposal_index: i64,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct FullProposalInfo {
    #[serde(flatten)]
    pub proposal: Proposal,
    #[serde(flatten)]
    pub voteplan: ProposalVotePlanCommon,
    #[serde(alias = "groupId")]
    pub group_id: String,
}

impl From<vit_servicing_station_lib::db::models::proposals::FullProposalInfo> for FullProposalInfo {
    fn from(val: vit_servicing_station_lib::db::models::proposals::FullProposalInfo) -> Self {
        Self {
            proposal: Proposal {
                internal_id: val.proposal.internal_id,
                proposal_id: val.proposal.proposal_id,
                proposal_title: val.proposal.proposal_title,
                proposal_funds: val.proposal.proposal_funds,
                proposal_url: val.proposal.proposal_url,
                proposal_impact_score: val.proposal.proposal_impact_score,
                chain_proposal_id: val.proposal.chain_proposal_id,
                chain_vote_options: VoteOptions(val.proposal.chain_vote_options.0),
                challenge_id: val.proposal.challenge_id,
            },
            voteplan: ProposalVotePlanCommon {
                chain_voteplan_id: val.voteplan.chain_voteplan_id,
                chain_proposal_index: val.voteplan.chain_proposal_index,
            },
            group_id: val.group_id,
        }
    }
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
            Proposal {
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
