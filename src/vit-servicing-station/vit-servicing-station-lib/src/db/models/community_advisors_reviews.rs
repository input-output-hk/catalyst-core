use crate::db::schema::community_advisors_reviews;

use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Insertable, Queryable)]
#[table_name = "community_advisors_reviews"]
pub struct AdvisorReview {
    pub id: i32,
    pub proposal_id: i32,
    pub rating_given: i32,
    pub assessor: String,
    pub note: String,
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
        }
    }

    pub fn pupulate_db_with_advisor_review(review: &AdvisorReview, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();
        diesel::insert_into(community_advisors_reviews::table)
            .values(review)
            .execute(&connection)
            .unwrap();
    }
}
