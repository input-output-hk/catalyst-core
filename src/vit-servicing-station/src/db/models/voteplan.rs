use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable)]
pub struct Voteplan {
    id: i32,
    chain_voteplan_id: String,
    chain_vote_start_time: String,
    chain_vote_end_time: String,
    chain_committee_end: String,
    chain_voteplan_payload: String,
    fund_id: i32,
}
