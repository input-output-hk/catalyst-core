use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ReviewRanking {
    Excellent,
    Good,
    FilteredOut,
    NotReviewedByVCA,
}

impl ReviewRanking {
    pub fn is_positive(&self) -> bool {
        matches!(self, Self::Excellent | Self::Good)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdvisorReview {
    pub id: i32,
    pub proposal_id: i32,
    pub assessor: String,
    pub impact_alignment_rating_given: i32,
    pub impact_alignment_note: String,
    pub feasibility_rating_given: i32,
    pub feasibility_note: String,
    pub auditability_rating_given: i32,
    pub auditability_note: String,
    pub ranking: ReviewRanking,
}
