use crate::db::{
    models::funds::Fund, schema::funds::dsl as funds_dsl, schema::funds::dsl::funds as funds_table,
};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use crate::v0::errors::HandleError;
use async_graphql::{Context, ErrorExtensions, FieldResult};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn funds<'ctx>(root: &QueryRoot, _ctx: &Context<'_>) -> FieldResult<Vec<Fund>> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(|e| HandleError::DatabaseError(e).extend())?;
    tokio::task::spawn_blocking(move || {
        funds_table
            .load::<Fund>(&db_conn)
            .map_err(|_| HandleError::InternalError("Error retrieving funds".to_string()).extend())
    })
    .await?
}

pub async fn fund<'ctx>(root: &QueryRoot, fund_id: i32, _ctx: &Context<'_>) -> FieldResult<Fund> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(|e| HandleError::DatabaseError(e).extend())?;
    tokio::task::spawn_blocking(move || {
        funds_table
            .filter(funds_dsl::id.eq(fund_id))
            .first::<Fund>(&db_conn)
            .map_err(|_| HandleError::NotFound(format!("id {}", fund_id)).extend())
    })
    .await?
}
