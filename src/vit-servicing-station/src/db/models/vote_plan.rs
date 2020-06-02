use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable)]
pub struct ChainVoteplan {
    chain_voteplan_id: String,
    chain_vote_start_time: String,
    chain_vote_end_time: String,
    chain_committee_end: String,
}
