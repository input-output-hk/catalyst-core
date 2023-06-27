use crate::utils;
use serde::Deserialize;
use std::path::PathBuf;

const ALLOCATED_TYPE: u32 = 1;
const NOT_ALLOCATED_TYPE: u32 = 0;

#[derive(Deserialize, Debug)]
pub struct ReviewsRow {
    #[serde(alias = "Review Type")]
    review_type: u32,
    #[serde(alias = "Impact / Alignment Rating")]
    alignment_rating: u32,
    #[serde(alias = "Feasibility Rating")]
    feasibility_rating: u32,
    #[serde(alias = "Auditability Rating")]
    auditability_rating: u32,
}

pub fn load_reviews_from_csv(path: &PathBuf) -> Result<Vec<ReviewsRow>, csv::Error> {
    utils::csv::load_data_from_csv::<_, b','>(path)
}
