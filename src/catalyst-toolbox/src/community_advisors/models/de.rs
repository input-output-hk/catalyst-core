use crate::utils::serde::deserialize_truthy_falsy;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AdvisorReviewRow {
    pub proposal_id: String,
    #[serde(alias = "Idea URL")]
    pub idea_url: String,
    #[serde(alias = "Assessor")]
    pub assessor: String,
    #[serde(alias = "Impact / Alignment Note")]
    pub impact_alignment_note: String,
    #[serde(alias = "Impact / Alignment Rating")]
    pub impact_alignment_rating: u8,
    #[serde(alias = "Feasibility Note")]
    pub feasibility_note: String,
    #[serde(alias = "Feasibility Rating")]
    pub feasibility_rating: u8,
    #[serde(alias = "Auditability Note")]
    pub auditability_note: String,
    #[serde(alias = "Auditability Rating")]
    pub auditability_rating: u8,
    #[serde(alias = "Excellent", deserialize_with = "deserialize_truthy_falsy")]
    excellent: bool,
    #[serde(alias = "Good", deserialize_with = "deserialize_truthy_falsy")]
    good: bool,
}

pub enum ReviewScore {
    Excellent,
    Good,
    FilteredOut,
    NA, // not reviewed by vCAs
}

impl AdvisorReviewRow {
    pub fn score(&self) -> ReviewScore {
        match (self.excellent, self.good) {
            (true, false) => ReviewScore::Excellent,
            (false, true) => ReviewScore::Good,
            (false, false) => ReviewScore::NA,
            _ => {
                // This should never happen, from the source of information a review could be either
                // Excellent or Good or not assessed. It cannot be both and it is considered
                // a malformed information input.
                panic!(
                    "Invalid combination of scores from assessor {} for proposal {}",
                    self.assessor, self.proposal_id
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReviewScore;
    use crate::community_advisors::models::AdvisorReviewRow;
    use crate::utils::csv as csv_utils;
    use rand::RngCore;
    use std::path::PathBuf;

    #[test]
    fn test_deserialize() {
        let file_path = PathBuf::from("./resources/testing/valid_assessments.csv");
        let data: Vec<AdvisorReviewRow> =
            csv_utils::load_data_from_csv::<_, b','>(&file_path).unwrap();
        assert_eq!(data.len(), 1);
    }

    impl AdvisorReviewRow {
        pub fn dummy(score: ReviewScore) -> Self {
            let (excellent, good) = match score {
                ReviewScore::Good => (false, true),
                ReviewScore::Excellent => (true, false),
                ReviewScore::NA => (false, false),
                _ => unimplemented!(),
            };

            AdvisorReviewRow {
                proposal_id: String::new(),
                idea_url: String::new(),
                assessor: rand::thread_rng().next_u64().to_string(),
                impact_alignment_note: String::new(),
                impact_alignment_rating: 0,
                feasibility_note: String::new(),
                feasibility_rating: 0,
                auditability_note: String::new(),
                auditability_rating: 0,
                excellent,
                good,
            }
        }
    }
}
