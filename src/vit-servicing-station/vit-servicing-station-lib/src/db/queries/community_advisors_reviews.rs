use crate::db::{
    models::community_advisors_reviews::AdvisorReview,
    schema::community_advisors_reviews::{self, dsl as reviews_dsl},
    DbConnection, DbConnectionPool,
};
use crate::v0::errors::HandleError;

use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_reviews_by_fund_id(
    id: i32,
    pool: &DbConnectionPool,
) -> Result<Vec<AdvisorReview>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        reviews_dsl::community_advisors_reviews
            .filter(reviews_dsl::proposal_id.eq(id))
            .load::<AdvisorReview>(&db_conn)
            .map_err(|_e| {
                HandleError::NotFound("Error loading community advisors reviews".to_string())
            })
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_advisor_reviews(
    reviews: &[AdvisorReview],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(community_advisors_reviews::table)
        .values(
            reviews
                .iter()
                .cloned()
                .map(|r| r.values())
                .collect::<Vec<_>>(),
        )
        .execute(db_conn)
}
