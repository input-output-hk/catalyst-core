use crate::db;
use crate::db::schema::proposals::dsl::proposals;
use async_graphql::*;
use diesel::prelude::*;
use diesel::Queryable;
use std::sync::Arc;

#[derive(Queryable)]
pub struct Proposal {
    pub id: i32,
    pub proposal_category: String,
    pub proposal_id: String,
    pub proposal_title: String,
    pub proposal_summary: String,
    pub proposal_problem: String,
    pub proposal_solution: String,
    pub proposal_funds: i32,
    pub proposal_url: String,
    pub proposal_files_url: String,
    pub proposer_name: String,
    pub proposer_contact: String,
    pub proposer_url: String,
    pub chain_proposal_id: String,
    pub chain_voteplan_id: String,
    pub chain_proposal_index: i32,
    pub chain_vote_start_time: i32,
    pub chain_vote_end_time: i32,
    pub chain_committee_end_time: i32,
    pub chain_vote_options: String,
}

#[async_graphql::Object]
impl Proposal {}

pub struct QueryRoot {
    pub db_connection_pool: Arc<db::DBConnectionPool>,
}

#[Object]
impl QueryRoot {
    #[field(desc = "Proposal information")]
    async fn proposals<'ctx>(&self, _ctx: &Context<'_>) -> Vec<Proposal> {
        let db_conn = self
            .db_connection_pool
            .get()
            .expect("Error connecting to database");
        proposals
            .load::<Proposal>(&db_conn)
            .expect("Error loading proposals")
    }
}
