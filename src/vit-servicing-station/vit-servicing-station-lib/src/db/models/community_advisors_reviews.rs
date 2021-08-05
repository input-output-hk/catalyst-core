use crate::db::schema::community_advisors_reviews;

use crate::db::Db;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReviewTag {
    Alignment,
    Verifiability,
    Feasibility,
    Impact,
    Auditability,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct AdvisorReview {
    pub id: i32,
    pub proposal_id: i32,
    pub rating_given: i32,
    pub assessor: String,
    pub note: String,
    pub tag: ReviewTag,
}

type AdvisorReviewRow = (
    // 0 -> id
    i32,
    // 1 -> proposal_id
    i32,
    // 2 -> rating_given
    i32,
    // 3 -> assessor
    String,
    // 4 -> note
    String,
    // 5 -> tag
    String,
);

impl ToString for ReviewTag {
    fn to_string(&self) -> String {
        match self {
            ReviewTag::Alignment => "Alignment",
            ReviewTag::Verifiability => "Verifiability",
            ReviewTag::Feasibility => "Feasibility",
            ReviewTag::Impact => "Impact",
            ReviewTag::Auditability => "Auditability",
        }
        .to_string()
    }
}

impl FromStr for ReviewTag {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "alignment" => Ok(Self::Alignment),
            "verifiability" => Ok(Self::Verifiability),
            "feasibility" => Ok(Self::Feasibility),
            "impact" => Ok(Self::Impact),
            "Auditability" => Ok(Self::Auditability),
            tag => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unrecognized review tag {}", tag),
            )),
        }
    }
}

impl Queryable<community_advisors_reviews::SqlType, Db> for AdvisorReview {
    type Row = AdvisorReviewRow;

    fn build(row: Self::Row) -> Self {
        Self {
            id: row.0,
            proposal_id: row.1,
            rating_given: row.2,
            assessor: row.3,
            note: row.4,
            tag: ReviewTag::from_str(&row.5).unwrap(),
        }
    }
}

impl Insertable<community_advisors_reviews::table> for AdvisorReview {
    #[allow(clippy::type_complexity)]
    type Values = (
        diesel::dsl::Eq<community_advisors_reviews::proposal_id, i32>,
        diesel::dsl::Eq<community_advisors_reviews::rating_given, i32>,
        diesel::dsl::Eq<community_advisors_reviews::assessor, String>,
        diesel::dsl::Eq<community_advisors_reviews::note, String>,
        diesel::dsl::Eq<community_advisors_reviews::tag, String>,
    );

    fn values(self) -> Self::Values {
        (
            community_advisors_reviews::proposal_id.eq(self.proposal_id),
            community_advisors_reviews::rating_given.eq(self.rating_given),
            community_advisors_reviews::assessor.eq(self.assessor),
            community_advisors_reviews::note.eq(self.note),
            community_advisors_reviews::tag.eq(self.tag.to_string()),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DbConnectionPool;
    use diesel::RunQueryDsl;

    pub fn get_test_advisor_review_with_proposal_id(proposal_id: i32) -> AdvisorReview {
        AdvisorReview {
            id: 0,
            proposal_id,
            rating_given: 500,
            assessor: "foo bar".to_string(),
            note: "no notes".to_string(),
            tag: ReviewTag::Alignment,
        }
    }

    pub fn pupulate_db_with_advisor_review(review: &AdvisorReview, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();
        diesel::insert_into(community_advisors_reviews::table)
            .values(review.clone().values())
            .execute(&connection)
            .unwrap();
    }
}
