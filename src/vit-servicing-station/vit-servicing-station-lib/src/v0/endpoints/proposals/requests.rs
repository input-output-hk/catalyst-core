use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProposalVoteplanIdAndIndexes {
    pub vote_plan_id: String,
    pub indexes: Vec<i64>,
}

pub type ProposalsByVoteplanIdAndIndex = Vec<ProposalVoteplanIdAndIndexes>;
