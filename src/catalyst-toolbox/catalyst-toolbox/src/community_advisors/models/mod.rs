mod de;

use serde::{Deserialize, Deserializer};

pub use de::{
    AdvisorReviewId, AdvisorReviewRow, ReviewRanking, VeteranAdvisorId, VeteranRankingRow,
};

pub enum ProposalStatus {
    Approved,
    NotApproved,
}

#[derive(Deserialize)]
pub struct ApprovedProposalRow {
    #[serde(rename(deserialize = "internal_id"))]
    pub proposal_id: String,
    #[serde(rename(deserialize = "meets_approval_threshold"))]
    pub status: ProposalStatus,
    pub requested_dollars: String,
}

impl<'de> Deserialize<'de> for ProposalStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let status: String = String::deserialize(deserializer)?;
        Ok(match status.to_lowercase().as_ref() {
            "yes" => ProposalStatus::Approved,
            _ => ProposalStatus::NotApproved,
        })
    }
}
