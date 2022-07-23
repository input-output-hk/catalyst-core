use crate::config::VoteTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct VotePlan {
    #[serde(default)]
    pub vote_time: VoteTime,
    #[serde(default)]
    pub private: bool,
}
