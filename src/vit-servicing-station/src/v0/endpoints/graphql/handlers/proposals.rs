use crate::db::{
    models::proposals::Proposal, views_schema::full_proposals_info::dsl as proposals_dsl,
    views_schema::full_proposals_info::dsl::full_proposals_info as proposals_table,
};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::Context;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn proposals<'ctx>(root: &QueryRoot, _ctx: &Context<'_>) -> Vec<Proposal> {
    let db_conn = root
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    tokio::task::spawn_blocking(move || {
        proposals_table
            .load::<Proposal>(&db_conn)
            .expect("Error loading proposals")
    })
    .await
    .expect("Error loading proposals")
}

pub async fn proposal<'ctx>(
    root: &QueryRoot,
    proposal_id: String,
    _ctx: &Context<'_>,
) -> async_graphql::FieldResult<Proposal> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(async_graphql::FieldError::from)?;
    tokio::task::spawn_blocking(move || {
        proposals_table
            .filter(proposals_dsl::proposal_id.eq(proposal_id))
            .first::<Proposal>(&db_conn)
            .map_err(async_graphql::FieldError::from)
    })
    .await?
    .map_err(async_graphql::FieldError::from)
}
