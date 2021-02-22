use crate::db::{
    models::proposals::FullProposalInfo, views_schema::full_proposals_info::dsl as proposals_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info as proposals_table,
};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use crate::v0::errors::HandleError;
use async_graphql::{Context, ErrorExtensions, FieldResult};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn proposals<'ctx>(
    root: &QueryRoot,
    _ctx: &Context<'_>,
) -> FieldResult<Vec<FullProposalInfo>> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(|e| HandleError::DatabaseError(e).extend())?;
    tokio::task::spawn_blocking(move || {
        proposals_table
            .load::<FullProposalInfo>(&db_conn)
            .map_err(|_| {
                HandleError::InternalError("Error retrieving proposals".to_string()).extend()
            })
    })
    .await?
}

pub async fn proposal<'ctx>(
    root: &QueryRoot,
    proposal_id: String,
    _ctx: &Context<'_>,
) -> FieldResult<FullProposalInfo> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(|e| HandleError::DatabaseError(e).extend())?;
    tokio::task::spawn_blocking(move || {
        proposals_table
            .filter(proposals_dsl::proposal_id.eq(proposal_id.clone()))
            .first::<FullProposalInfo>(&db_conn)
            .map_err(|_| HandleError::NotFound(format!("proposal id {}", proposal_id)).extend())
    })
    .await?
}
