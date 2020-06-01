use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i32,
    pub proposal_category: String,
    pub proposal_id: String,
    pub proposal_title: String,
    pub proposal_summary: String,
    pub proposal_problem: String,
    pub proposal_solution: String,
    pub proposal_funds: i64,
    pub proposal_url: String,
    pub proposal_files_url: String,
    pub proposer_name: String,
    pub proposer_contact: String,
    pub proposer_url: String,
    pub chain_proposal_id: String,
    pub chain_voteplan_id: String,
    pub chain_proposal_index: i64,
    pub chain_vote_start_time: i64,
    pub chain_vote_end_time: i64,
    pub chain_committee_end_time: i64,
    pub chain_vote_options: String,
}
