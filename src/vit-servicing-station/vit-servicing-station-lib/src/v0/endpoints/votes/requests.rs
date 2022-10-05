use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteCasterAndVoteplanId {
    pub vote_plan_id: String,
    pub caster: String,
}
