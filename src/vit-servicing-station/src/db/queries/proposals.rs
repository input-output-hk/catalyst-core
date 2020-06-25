use crate::db::{
    models::proposals::Proposal, views_schema::full_proposals_info::dsl as full_proposal_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info, DBConnectionPool,
};
use crate::v0::errors::HandleError;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn query_all_proposals(pool: &DBConnectionPool) -> Result<Vec<Proposal>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .load::<Proposal>(&db_conn)
            .map_err(|_e| HandleError::NotFound("proposals".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn query_proposal_by_id(
    id: i32,
    pool: &DBConnectionPool,
) -> Result<Proposal, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .filter(full_proposal_dsl::id.eq(id))
            .first::<Proposal>(&db_conn)
            .map_err(|_e| HandleError::NotFound(format!("proposal with id {}", id)))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}
