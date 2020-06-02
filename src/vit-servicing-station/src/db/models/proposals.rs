use crate::db::{schema::proposals, DB};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Category {
    pub category_id: String,
    pub category_name: String,
    pub category_description: String,
}

#[derive(Serialize, Deserialize)]
pub struct Proposer {
    pub proposer_name: String,
    pub proposer_email: String,
    pub proposer_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct Proposal {
    pub proposal_id: String,
    pub category: Category,
    pub proposal_title: String,
    pub proposal_summary: String,
    pub proposal_problem: String,
    pub proposal_solution: String,
    pub proposal_funds: i64,
    pub proposal_url: String,
    pub proposal_files_url: String,
    pub proposer: Proposer,
    pub chain_proposal_id: String,
    pub chain_voteplan_id: String,
    pub chain_proposal_index: i64,
    pub chain_vote_start_time: i64,
    pub chain_vote_end_time: i64,
    pub chain_committee_end_time: i64,
    pub chain_vote_options: String,
}

impl Queryable<proposals::SqlType, DB> for Proposal {
    // The row is the row, for now it cannot be any other type, may change when the DB schema changes
    #[allow(clippy::type_complexity)]
    type Row = (
        // 0 -> category_name
        String,
        // 1-> proposal_id
        String,
        // 2 -> proposal_title
        String,
        // 3 -> proposal_summary
        String,
        // 4 -> proposal_problem
        String,
        // 5 -> proposal_solution
        String,
        // 6 -> proposal_funds
        i64,
        // 7 -> proposal_url
        String,
        // 8 -> proposal_files_url,
        String,
        // 9 -> proposer_name
        String,
        // 10 -> proposer_contact
        String,
        // 11 -> proposer_url
        String,
        // 12 -> chain_proposal_id
        String,
        // 13 -> chain_voteplan_id
        String,
        // 14 -> chain_proposal_index
        i64,
        // 15 -> chain_vote_start_time
        i64,
        // 16 -> chain_vote_end_time
        i64,
        // 17 -> chain_committee_end_time
        i64,
        // 18 -> chain_vote_options
        String,
    );

    fn build(row: Self::Row) -> Self {
        Proposal {
            category: Category {
                category_name: row.0,
                category_id: "".to_string(),
                category_description: "".to_string(),
            },
            proposal_id: row.1,
            proposal_title: row.2,
            proposal_summary: row.3,
            proposal_problem: row.4,
            proposal_solution: row.5,
            proposal_funds: row.6,
            proposal_url: row.7,
            proposal_files_url: row.8,
            proposer: Proposer {
                proposer_name: row.9,
                proposer_email: row.10,
                proposer_url: row.11,
            },
            chain_proposal_id: row.12,
            chain_voteplan_id: row.13,
            chain_proposal_index: row.14,
            chain_vote_start_time: row.15,
            chain_vote_end_time: row.16,
            chain_committee_end_time: row.17,
            chain_vote_options: row.18,
        }
    }
}
