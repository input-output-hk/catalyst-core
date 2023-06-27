use crate::utils;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct ReviewsRow {
    id: u32,
    #[serde(alias = "Challenge")]
    challenge: String,
    #[serde(alias = "Idea Title")]
    idea_title: String,
    #[serde(alias = "Idea URL")]
    idea_url: String,
    #[serde(alias = "Reviewer")]
    reviewer: String,
    #[serde(alias = "Level")]
    level: u32,
    #[serde(alias = "Review Type")]
    review_type: u32,
    proposal_id: u32,
    challenge_id: u32,
    #[serde(alias = "Impact / Alignment Note")]
    alignment_note: String,
    #[serde(alias = "Impact / Alignment Rating")]
    alignment_rating: u32,
    #[serde(alias = "Feasibility Note")]
    feasibility_note: String,
    #[serde(alias = "Feasibility Rating")]
    feasibility_rating: String,
    #[serde(alias = "Auditability Note")]
    auditability_note: String,
    #[serde(alias = "Auditability Rating")]
    auditability_rating: u32,
}

fn load_reviews_from_csv(path: &PathBuf) -> Result<Vec<ReviewsRow>, csv::Error> {
    utils::csv::load_data_from_csv::<_, b','>(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_csv_test() {
        let data = load_reviews_from_csv(&PathBuf::from("src/proposal_score/reviews-example.csv"));
        println!("{:?}", data);
    }
}
