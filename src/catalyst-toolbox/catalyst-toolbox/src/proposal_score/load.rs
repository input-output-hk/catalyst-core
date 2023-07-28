use super::{AlignmentReviews, AuditabilityReviews, FeasibilityReviews, ProposalId, Review};
use crate::utils;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error("Invalid review type value, should be either 0 or 1, provided: {0}")]
    InvalidReviewType(u32),
    #[error("Invalid proposal data: {0}")]
    InvalidProposalData(String),
}

const ALLOCATED_TYPE: u32 = 1;
const NOT_ALLOCATED_TYPE: u32 = 0;

/// Represents a cvs's row, as an example of csv file in `tets_data/reviews-example.csv`
/// All serde aliases are used to be corresponded with the csv's column names
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
    path: &Path,
) -> Result<HashMap<ProposalId, (AlignmentReviews, FeasibilityReviews, AuditabilityReviews)>, Error>
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
                AlignmentReviews(Vec::new()),
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

fn retrieve_proposal_id(proposal: &serde_json::Value) -> Result<ProposalId, Error> {
    Ok(ProposalId(
        proposal
            .get("proposal_id")
            .ok_or_else(|| {
                Error::InvalidProposalData("does not have \"proposal_id\" field".to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                Error::InvalidProposalData(
                    "invalid \"proposal_id\" field type, not a string".to_string(),
                )
            })?
            .parse()
            .map_err(|_| {
                Error::InvalidProposalData(
                    "invalid \"proposal_id\" field type, not a string encoded int".to_string(),
                )
            })?,
    ))
}

pub fn load_proposals_from_json(
    path: &Path,
) -> Result<HashMap<ProposalId, serde_json::Value>, Error> {
    let proposals: Vec<serde_json::Value> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    proposals
        .into_iter()
        .map(
            |proposal| -> Result<(ProposalId, serde_json::Value), Error> {
                Ok((retrieve_proposal_id(&proposal)?, proposal))
            },
        )
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn load_reviews_test() {
        let file = PathBuf::from("src/proposal_score/test_data/reviews-example.csv");
        let res = load_reviews_from_csv(&file).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn retrieve_proposal_id_test() {
        let proposal = serde_json::json!({
            "proposal_id": "32",
        });
        let proposal_id = retrieve_proposal_id(&proposal).unwrap();
        assert_eq!(proposal_id, ProposalId(32));
    }

    #[test]
    fn load_proposals_test() {
        let file = PathBuf::from("src/proposal_score/test_data/proposals.json");
        let res = load_proposals_from_json(&file).unwrap();
        println!("{:?}", res);
    }
}
