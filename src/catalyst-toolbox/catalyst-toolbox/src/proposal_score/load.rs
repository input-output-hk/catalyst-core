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

// impl TryInto<super::Review> for ReviewsRow {
//     type Error = String;
//     fn try_into(self) -> Result<super::Review, Self::Error> {
//         let allocated = match self.review_type {
//             ALLOCATED_TYPE => true,
//             NOT_ALLOCATED_TYPE => false,
//             _ => {
//                 return Err("Invalid review type".to_string());
//             }
//         };
//         Ok(super::Review {
//             allocated,
//             alignment_rating: self.alignment_rating,
//             feasibility_rating: self.feasibility_rating,
//             auditability_rating: self.auditability_rating,
//         })
//     }
// }

pub fn load_reviews_from_csv(path: &PathBuf) -> Result<Vec<ReviewsRow>, csv::Error> {
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
