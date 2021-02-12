use crate::{
    db::{
        models::{challenges::Challenge, proposals::Proposal},
        schema::challenges::{self, dsl as challenges_dsl},
        views_schema::full_proposals_info::dsl as proposals_dsl,
        DBConnection, DBConnectionPool,
    },
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryResult, RunQueryDsl};

pub async fn query_all_challenges(pool: &DBConnectionPool) -> Result<Vec<Challenge>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        challenges_dsl::challenges
            .load::<Challenge>(&db_conn)
            .map_err(|_| HandleError::InternalError("Error retrieving challenges".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenge_by_id(
    id: i32,
    pool: &DBConnectionPool,
) -> Result<Challenge, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(challenges_dsl::challenges, challenges_dsl::id.eq(id))
            .first::<Challenge>(&db_conn)
            .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenges_by_fund_id(
    fund_id: i32,
    pool: &DBConnectionPool,
) -> Result<Vec<Challenge>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(
            challenges_dsl::challenges,
            challenges_dsl::fund_id.eq(fund_id),
        )
        .load::<Challenge>(&db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading challenges for fund id".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenge_proposals_by_id(
    id: i32,
    pool: &DBConnectionPool,
) -> Result<Vec<Proposal>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        diesel::QueryDsl::filter(
            proposals_dsl::full_proposals_info,
            proposals_dsl::challenge_id.eq(id),
        )
        .load::<Proposal>(&db_conn)
        .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_challenges(
    challenges_slice: &[Challenge],
    db_conn: &DBConnection,
) -> QueryResult<usize> {
    diesel::insert_into(challenges::table)
        .values(
            challenges_slice
                .iter()
                .cloned()
                .map(|challenge| challenge.values())
                .collect::<Vec<_>>(),
        )
        .execute(db_conn)
}
