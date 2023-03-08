use serde::Serialize;

#[derive(Serialize, Clone, Default)]
pub struct ProposalCategory {
    category_id: String,
    category_name: String,
    category_description: String,
}

#[derive(Serialize, Clone, Default)]
pub struct Proposer {
    proposer_name: String,
    proposer_email: String,
    proposer_url: String,
}

#[derive(Serialize, Clone, Default)]
pub struct Proposal {
    internal_id: i32,
    proposal_id: String,
    proposal_category: ProposalCategory,
    proposal_title: String,
    proposal_summary: String,
    proposal_public_key: String,
    proposal_funds: i64,
    proposal_url: String,
    proposal_files: String,
    proposal_extra_fields: String,
    proposer: Proposer,
    chain_proposal_id: String,
    // TODO: need to add chain_vote_options field, currently unknown type
    chain_vote_start_time: String,
    chain_vote_end_time: String,
    chain_committee_end_time: String,
    chain_voteplan_payload: String,
    chain_voteplan_id: String,
    chain_proposal_index: i64,
}

pub trait ProposalDb {
    fn get_proposals_by_voter_group_id(&self, voter_group_id: String) -> Vec<Proposal>;
    fn get_proposal_by_and_by_voter_group_id(&self, id: i32, voter_group_id: String) -> Proposal;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposal_category_json_test() {
        let proposal_category = ProposalCategory {
            category_id: "category_id".to_string(),
            category_name: "category_name".to_string(),
            category_description: "category_description".to_string(),
        };
        let json = serde_json::to_string(&proposal_category).unwrap();
        assert_eq!(
            json,
            r#"{"category_id":"category_id","category_name":"category_name","category_description":"category_description"}"#
        );
    }

    #[test]
    fn proposer_json_test() {
        let proposer = Proposer {
            proposer_name: "proposer_name".to_string(),
            proposer_email: "proposer_email".to_string(),
            proposer_url: "proposer_url".to_string(),
        };
        let json = serde_json::to_string(&proposer).unwrap();
        assert_eq!(
            json,
            r#"{"proposer_name":"proposer_name","proposer_email":"proposer_email","proposer_url":"proposer_url"}"#
        );
    }

    #[test]
    fn proposal_json_test() {
        let proposal = Proposal {
            internal_id: 0,
            proposal_id: "proposal_id".to_string(),
            proposal_category: ProposalCategory {
                category_id: "category_id".to_string(),
                category_name: "category_name".to_string(),
                category_description: "category_description".to_string(),
            },
            proposal_title: "proposal_title".to_string(),
            proposal_summary: "proposal_summary".to_string(),
            proposal_public_key: "proposal_public_key".to_string(),
            proposal_funds: 0,
            proposal_url: "proposal_url".to_string(),
            proposal_files: "proposal_files".to_string(),
            proposal_extra_fields: "proposal_extra_fields".to_string(),
            proposer: Proposer {
                proposer_name: "proposer_name".to_string(),
                proposer_email: "proposer_email".to_string(),
                proposer_url: "proposer_url".to_string(),
            },
            chain_proposal_id: "chain_proposal_id".to_string(),
            chain_vote_start_time: "chain_vote_start_time".to_string(),
            chain_vote_end_time: "chain_vote_end_time".to_string(),
            chain_committee_end_time: "chain_committee_end_time".to_string(),
            chain_voteplan_payload: "chain_voteplan_payload".to_string(),
            chain_voteplan_id: "chain_voteplan_id".to_string(),
            chain_proposal_index: 0,
        };
        let json = serde_json::to_string(&proposal).unwrap();
        assert_eq!(
            json,
            r#"{"internal_id":0,"proposal_id":"proposal_id","proposal_category":{"category_id":"category_id","category_name":"category_name","category_description":"category_description"},"proposal_title":"proposal_title","proposal_summary":"proposal_summary","proposal_public_key":"proposal_public_key","proposal_funds":0,"proposal_url":"proposal_url","proposal_files":"proposal_files","proposal_extra_fields":"proposal_extra_fields","proposer":{"proposer_name":"proposer_name","proposer_email":"proposer_email","proposer_url":"proposer_url"},"chain_proposal_id":"chain_proposal_id","chain_vote_start_time":"chain_vote_start_time","chain_vote_end_time":"chain_vote_end_time","chain_committee_end_time":"chain_committee_end_time","chain_voteplan_payload":"chain_voteplan_payload","chain_voteplan_id":"chain_voteplan_id","chain_proposal_index":0}"#
        );
    }
}
