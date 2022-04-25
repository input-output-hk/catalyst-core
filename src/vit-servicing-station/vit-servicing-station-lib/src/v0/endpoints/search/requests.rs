use serde::{Deserialize, Serialize};

use crate::db::models::{challenges::Challenge, proposals::FullProposalInfo};

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchResponse {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub enum SearchTable {
    Challenge,
    Proposal,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub enum SearchColumn {
    ChallengeTitle,
    ChallengeType,
    ChallengeDesc,
    ProposalAuthor,
    ProposalTitle,
    ProposalSummary,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub enum SearchSort {
    ChallengeTitle,
    ProposalFunds,
    ProposalAdvisor,
    ProposalTitle,
    Random,
    Index,
}

impl Default for SearchSort {
    fn default() -> Self {
        Self::Index
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
pub struct SearchRequest {
    pub table: SearchTable,
    pub column: SearchColumn,
    #[serde(default)]
    pub sort: SearchSort,
    pub query: String,
}
