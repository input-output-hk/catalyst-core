use crate::{
    db::{
        models::{challenges::Challenge, proposals::Proposal},
        schema::challenges::{self, dsl as challenges_dsl},
        views_schema::full_proposals_info::dsl as proposals_dsl,
        DbConnection, DbConnectionPool,
    },
    execute_q, q,
    v0::errors::HandleError,
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, QueryResult, RunQueryDsl};

pub async fn query_all_challenges(pool: &DbConnectionPool) -> Result<Vec<Challenge>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            challenges_dsl::challenges
                .order_by(challenges_dsl::internal_id.asc())
                .load::<Challenge>(&db_conn)
        )
        .map_err(|_| HandleError::InternalError("Error retrieving challenges".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenge_by_id(
    id: i32,
    pool: &DbConnectionPool,
) -> Result<Challenge, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            challenges_dsl::challenges
                .filter(challenges_dsl::id.eq(id))
                .first::<Challenge>(&db_conn)
        )
        .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenges_by_fund_id(
    fund_id: i32,
    pool: &DbConnectionPool,
) -> Result<Vec<Challenge>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            challenges_dsl::challenges
                .filter(challenges_dsl::fund_id.eq(fund_id))
                .order_by(challenges_dsl::internal_id.asc())
                .load::<Challenge>(&db_conn)
        )
        .map_err(|_e| HandleError::NotFound("Error loading challenges for fund id".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenge_proposals_by_id(
    id: i32,
    pool: &DbConnectionPool,
) -> Result<Vec<Proposal>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            proposals_dsl::full_proposals_info
                .filter(proposals_dsl::challenge_id.eq(id))
                .load::<Proposal>(&db_conn)
        )
        .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_challenge_proposals_by_id_and_group_id(
    id: i32,
    voter_group_id: String,
    pool: &DbConnectionPool,
) -> Result<Vec<Proposal>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        q!(
            db_conn,
            proposals_dsl::full_proposals_info
                .filter(proposals_dsl::challenge_id.eq(id))
                .filter(proposals_dsl::group_id.eq(voter_group_id))
                .load::<Proposal>(&db_conn)
        )
        .map_err(|_e| HandleError::NotFound("Error loading challenge".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn batch_insert_challenges(
    challenges: &[<Challenge as Insertable<challenges::table>>::Values],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    execute_q!(
        db_conn,
        diesel::insert_into(challenges::table).values(challenges)
    )
}
