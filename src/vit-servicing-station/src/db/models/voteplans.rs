use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable)]
pub struct Voteplan {
    pub id: i32,
    pub chain_voteplan_id: String,
    pub chain_vote_start_time: String,
    pub chain_vote_end_time: String,
    pub chain_committee_end: String,
    pub chain_voteplan_payload: String,
    pub fund_id: i32,
}
