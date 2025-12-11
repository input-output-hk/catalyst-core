#![allow(non_local_definitions)]

use crate::db::schema::community_advisors_reviews;

use diesel::prelude::*;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql},
    sql_types::Integer,
    FromSqlRow, Insertable, Queryable,
};
use serde::{Deserialize, Serialize};

#[allow(non_local_definitions)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Serialize, FromSqlRow, Deserialize)]
pub enum ReviewRanking {
    Excellent = 0,
    Good = 1,
    FilteredOut = 2,
    NA = 3, // not reviewed by vCAs
}

impl<DB> FromSql<Integer, DB> for ReviewRanking
where
    DB: Backend,
    i32: FromSql<Integer, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> deserialize::Result<Self> {
        match i32::from_sql(bytes)? {
            0 => Ok(ReviewRanking::Excellent),
            1 => Ok(ReviewRanking::Good),
            2 => Ok(ReviewRanking::FilteredOut),
            3 => Ok(ReviewRanking::NA),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

#[allow(non_local_definitions)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable)]
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

impl Insertable<community_advisors_reviews::table> for AdvisorReview {
    #[allow(clippy::type_complexity)]
    type Values = (
        diesel::dsl::Eq<community_advisors_reviews::proposal_id, i32>,
        diesel::dsl::Eq<community_advisors_reviews::assessor, String>,
        diesel::dsl::Eq<community_advisors_reviews::impact_alignment_rating_given, i32>,
        diesel::dsl::Eq<community_advisors_reviews::impact_alignment_note, String>,
        diesel::dsl::Eq<community_advisors_reviews::feasibility_rating_given, i32>,
        diesel::dsl::Eq<community_advisors_reviews::feasibility_note, String>,
        diesel::dsl::Eq<community_advisors_reviews::auditability_rating_given, i32>,
        diesel::dsl::Eq<community_advisors_reviews::auditability_note, String>,
        diesel::dsl::Eq<community_advisors_reviews::ranking, i32>,
    );

    fn values(self) -> Self::Values {
        (
            community_advisors_reviews::proposal_id.eq(self.proposal_id),
            community_advisors_reviews::assessor.eq(self.assessor),
            community_advisors_reviews::impact_alignment_rating_given
                .eq(self.impact_alignment_rating_given),
            community_advisors_reviews::impact_alignment_note.eq(self.impact_alignment_note),
            community_advisors_reviews::feasibility_rating_given.eq(self.feasibility_rating_given),
            community_advisors_reviews::feasibility_note.eq(self.feasibility_note),
            community_advisors_reviews::auditability_rating_given
                .eq(self.auditability_rating_given),
            community_advisors_reviews::auditability_note.eq(self.auditability_note),
            community_advisors_reviews::ranking.eq(self.ranking as i32),
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
            assessor: "foo bar".to_string(),
            impact_alignment_rating_given: 0,
            impact_alignment_note: "impact note".to_string(),
            feasibility_rating_given: 0,
            feasibility_note: "feasibility note".to_string(),
            auditability_rating_given: 0,
            auditability_note: "auditability".to_string(),
            ranking: ReviewRanking::Good,
        }
    }

    pub fn populate_db_with_advisor_review(review: &AdvisorReview, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();
        diesel::insert_into(community_advisors_reviews::table)
            .values(review.clone().values())
            .execute(&connection)
            .unwrap();
    }
}
