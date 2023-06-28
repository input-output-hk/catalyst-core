use super::{AligmentReviews, AuditabilityReviews, FeasibilityReviews, ProposalId, Review};
use crate::utils;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("Invalid review type value, should be either 0 or 1, provided: {0}")]
    InvalidReviewType(u32),
}

const ALLOCATED_TYPE: u32 = 1;
const NOT_ALLOCATED_TYPE: u32 = 0;

#[derive(Deserialize, Debug)]
pub struct ReviewsRow {
    proposal_id: i32,
    #[serde(alias = "Review Type")]
    review_type: u32,
    #[serde(alias = "Impact / Alignment Rating")]
    alignment_rating: u32,
    #[serde(alias = "Feasibility Rating")]
    feasibility_rating: u32,
    #[serde(alias = "Auditability Rating")]
    auditability_rating: u32,
}

pub fn load_reviews_from_csv(
    path: &PathBuf,
) -> Result<HashMap<ProposalId, (AligmentReviews, FeasibilityReviews, AuditabilityReviews)>, Error>
{
    let rows: Vec<ReviewsRow> = utils::csv::load_data_from_csv::<_, b','>(path)?;
    let mut reviews_per_proposal = HashMap::new();
    for row in rows {
        let allocated = match row.review_type {
            ALLOCATED_TYPE => true,
            NOT_ALLOCATED_TYPE => false,
            review_type => {
                return Err(Error::InvalidReviewType(review_type));
            }
        };
        let reviews = reviews_per_proposal
            .entry(ProposalId(row.proposal_id))
            .or_insert((
                AligmentReviews(Vec::new()),
                FeasibilityReviews(Vec::new()),
                AuditabilityReviews(Vec::new()),
            ));
        reviews.0 .0.push(Review {
            rating: row.alignment_rating,
            allocated,
        });
        reviews.1 .0.push(Review {
            rating: row.feasibility_rating,
            allocated,
        });
        reviews.2 .0.push(Review {
            rating: row.auditability_rating,
            allocated,
        });
    }

    Ok(reviews_per_proposal)
}

#[test]
#[ignore]
fn load_test() {
    let file = PathBuf::from("src/proposal_score/reviews-example.csv");

    let res = load_reviews_from_csv(&file).unwrap();
    println!("{:?}", res);
}
