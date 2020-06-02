use crate::db;
use crate::db::models::proposals::{Category, Proposal, Proposer};
use std::sync::Arc;

#[async_graphql::Object]
impl Category {
    pub async fn category_id(&self) -> &str {
        &self.category_id
    }

    pub async fn category_name(&self) -> &str {
        &self.category_name
    }

    pub async fn category_description(&self) -> &str {
        &self.category_description
    }
}

#[async_graphql::Object]
impl Proposer {
    pub async fn proposer_name(&self) -> &str {
        &self.proposer_name
    }

    pub async fn proposer_email(&self) -> &str {
        &self.proposer_email
    }

    pub async fn proposer_url(&self) -> &str {
        &self.proposer_url
    }
}

pub struct QueryRoot {
    pub db_connection_pool: Arc<db::DBConnectionPool>,
}

#[async_graphql::Object]
impl Proposal {
    pub async fn category(&self) -> &Category {
        &self.category
    }

    pub async fn proposal_id(&self) -> &str {
        &self.proposal_id
    }

    pub async fn proposal_title(&self) -> &str {
        &self.proposal_title
    }

    pub async fn proposal_summary(&self) -> &str {
        &self.proposal_summary
    }

    pub async fn proposal_problem(&self) -> &str {
        &self.proposal_problem
    }

    pub async fn proposal_solution(&self) -> &str {
        &self.proposal_solution
    }

    pub async fn proposal_funds(&self) -> i64 {
        self.proposal_funds
    }

    pub async fn proposal_url(&self) -> &str {
        &self.proposal_url
    }

    pub async fn proposal_files_url(&self) -> &str {
        &self.proposal_files_url
    }

    pub async fn proposer(&self) -> &Proposer {
        &self.proposer
    }

    pub async fn chain_proposal_id(&self) -> &str {
        &self.chain_proposal_id
    }

    pub async fn chain_voteplan_id(&self) -> &str {
        &self.chain_voteplan_id
    }

    pub async fn chain_proposal_index(&self) -> i64 {
        self.chain_proposal_index
    }

    pub async fn chain_vote_start_time(&self) -> i64 {
        self.chain_vote_start_time
    }

    pub async fn chain_vote_end_time(&self) -> i64 {
        self.chain_vote_end_time
    }

    pub async fn chain_committee_end_time(&self) -> i64 {
        self.chain_committee_end_time
    }

    pub async fn chain_vote_options(&self) -> &str {
        &self.chain_vote_options
    }
}
