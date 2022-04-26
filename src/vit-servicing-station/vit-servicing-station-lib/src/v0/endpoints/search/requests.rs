use serde::{Deserialize, Serialize};

use crate::db::models::{challenges::Challenge, proposals::FullProposalInfo};

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)] // should serialize as if it is either a `Vec<Challenge>` or `Vec<FullProposalInfo>`
pub enum SearchResponse {
    Challenge(Vec<Challenge>),
    Proposal(Vec<FullProposalInfo>),
}

impl SearchResponse {
    pub fn reverse(&mut self) {
        match self {
            Self::Challenge(vec) => vec.reverse(),
            Self::Proposal(vec) => vec.reverse(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "kebab-case")]
pub enum SearchTable {
    Challenge,
    Proposal,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "kebab-case")]
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
#[serde(rename_all = "kebab-case")]
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

fn fals() -> bool {
    false
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(rename_all = "kebab-case")]
pub struct SearchRequest {
    pub table: SearchTable,
    pub column: SearchColumn,
    #[serde(default)]
    pub sort: SearchSort,
    pub query: String,
    #[serde(default = "fals")]
    pub reverse: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_sort_is_optional() {
        let req = r#"{"table": "challenge", "column": "challenge-title", "query": "query"}"#;
        serde_json::from_str::<SearchRequest>(req).unwrap();
    }
}
