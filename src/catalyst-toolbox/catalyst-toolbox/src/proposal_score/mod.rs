pub mod load;
pub mod store;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProposalId(pub i32);

#[derive(Debug)]
pub struct AligmentReviews(pub Vec<Review>);
pub struct AligmentScore(f64);
#[derive(Debug)]
pub struct FeasibilityReviews(pub Vec<Review>);
pub struct FeasibilityScore(f64);
#[derive(Debug)]
pub struct AuditabilityReviews(pub Vec<Review>);
pub struct AuditabilityScore(f64);

pub fn calc_score(
    allocated_weight: f64,
    not_allocated_weight: f64,
    aligment_reviews: &AligmentReviews,
    feasibility_reviews: &FeasibilityReviews,
    auditability_review: &AuditabilityReviews,
) -> Result<(AligmentScore, FeasibilityScore, AuditabilityScore), Error> {
    let aligment_score = AligmentScore(weighted_avarage_score(
        allocated_weight,
        not_allocated_weight,
        &aligment_reviews.0,
    )?);
    let feasibility_score = FeasibilityScore(weighted_avarage_score(
        allocated_weight,
        not_allocated_weight,
        &feasibility_reviews.0,
    )?);
    let auditability_score = AuditabilityScore(weighted_avarage_score(
        allocated_weight,
        not_allocated_weight,
        &auditability_review.0,
    )?);

    Ok((aligment_score, feasibility_score, auditability_score))
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid allocated or not allocated weights, the sum of them should be less than 1.0. Got {0} and {1}")]
    InvalidWeights(f64, f64),
}

#[derive(Debug)]
pub struct Review {
    rating: u32,
    allocated: bool,
}

fn review_weight(weight: f64, reviews_amount: usize) -> f64 {
    weight / reviews_amount as f64
}

/// weighted average score calculation
fn weighted_avarage_score(
    allocated_weight: f64,
    not_allocated_weight: f64,
    reviews: &[Review],
) -> Result<f64, Error> {
    if allocated_weight + not_allocated_weight > 1.0 {
        return Err(Error::InvalidWeights(
            allocated_weight,
            not_allocated_weight,
        ));
    }

    let mut allocated_count = 0;
    let mut not_allocated_count = 0;

    let mut total_allocated_rating = 0;
    let mut total_not_allocated_rating = 0;
    for review in reviews {
        if review.allocated {
            allocated_count += 1;

            total_allocated_rating += review.rating;
        } else {
            not_allocated_count += 1;

            total_not_allocated_rating += review.rating;
        }
    }

    let allocated_weight = review_weight(allocated_weight, allocated_count);
    let not_allocated_weight = review_weight(not_allocated_weight, not_allocated_count);

    let res = (total_allocated_rating as f64 * allocated_weight
        + total_not_allocated_rating as f64 * not_allocated_weight)
        / (allocated_weight * allocated_count as f64
            + not_allocated_weight * not_allocated_count as f64);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_weight_test() {
        let total_weight = 0.2;
        let reviews_amount = 5;
        let result = review_weight(total_weight, reviews_amount);
        assert_eq!(result, 0.04);

        let total_weight = 0.8;
        let reviews_amount = 2;
        let result = review_weight(total_weight, reviews_amount);
        assert_eq!(result, 0.4);
    }

    #[test]
    fn weighted_score_test() {
        let allocated_weight = 0.8;
        let not_allocated_weight = 0.2;

        let reviews = vec![
            Review {
                rating: 5,
                allocated: false,
            },
            Review {
                rating: 5,
                allocated: false,
            },
            Review {
                rating: 5,
                allocated: false,
            },
            Review {
                rating: 5,
                allocated: false,
            },
            Review {
                rating: 5,
                allocated: false,
            },
            Review {
                rating: 2,
                allocated: true,
            },
            Review {
                rating: 2,
                allocated: true,
            },
        ];

        let result =
            weighted_avarage_score(allocated_weight, not_allocated_weight, &reviews).unwrap();
        assert_eq!(result, 2.6);

        assert!(weighted_avarage_score(0.5, 0.6, &[]).is_err());
    }

    #[test]
    fn full_test() {
        let allocated_weight = 0.8;
        let not_allocated_weight = 0.2;

        let db = std::path::PathBuf::from("src/proposal_score/test_data/fund9.sqlite3");
        let reviews = std::path::PathBuf::from("src/proposal_score/test_data/reviews-example.csv");

        let reviews = load::load_reviews_from_csv(&reviews).unwrap();

        for (proposal_id, (aligment_reviews, feasibility_reviews, auditability_reviews)) in reviews
        {
            let (aligment_score, feasibility_score, auditability_score) = calc_score(
                allocated_weight,
                not_allocated_weight,
                &aligment_reviews,
                &feasibility_reviews,
                &auditability_reviews,
            )
            .unwrap();

            store::store_scores_in_sqllite_db(
                &db,
                proposal_id,
                aligment_score,
                feasibility_score,
                auditability_score,
            )
            .unwrap();
        }
    }
}
