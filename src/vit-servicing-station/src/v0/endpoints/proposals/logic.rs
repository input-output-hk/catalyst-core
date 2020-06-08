use crate::db::{
    models::proposals::Proposal, views_schema::full_proposals_info::dsl as full_proposal_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info,
};
use crate::v0::{context::SharedContext, errors::HandleError};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn get_all_proposals(context: SharedContext) -> Result<Vec<Proposal>, HandleError> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .load::<Proposal>(&db_conn)
            .map_err(|_e| HandleError::NotFound("Error loading proposals".to_string()))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn get_proposal(id: i32, context: SharedContext) -> Result<Proposal, HandleError> {
    let db_conn = context
        .read()
        .await
        .db_connection_pool
        .get()
        .map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        full_proposals_info
            .filter(full_proposal_dsl::id.eq(id))
            .first::<Proposal>(&db_conn)
            .map_err(|_e| HandleError::NotFound(format!("Error loading proposal with id {}", id)))
    })
    .await
    .map_err(|_e| HandleError::InternalError("Error executing request".to_string()))?
}
