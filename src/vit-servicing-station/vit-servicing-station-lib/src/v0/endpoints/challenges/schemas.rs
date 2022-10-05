use crate::db::models::{challenges::Challenge, proposals::Proposal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ChallengeWithProposals {
    #[serde(flatten)]
    pub challenge: Challenge,
    pub proposals: Vec<Proposal>,
}
