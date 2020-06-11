use crate::db::{
    models::funds::Fund, schema::funds::dsl as funds_dsl, schema::funds::dsl::funds as funds_table,
};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::{Context, FieldResult};
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};

pub async fn funds<'ctx>(root: &QueryRoot, _ctx: &Context<'_>) -> FieldResult<Vec<Fund>> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(async_graphql::FieldError::from)?;
    tokio::task::spawn_blocking(move || {
        funds_table
            .load::<Fund>(&db_conn)
            .map_err(async_graphql::FieldError::from)
    })
    .await?
    .map_err(async_graphql::FieldError::from)
}

pub async fn fund<'ctx>(root: &QueryRoot, id: i32, _ctx: &Context<'_>) -> FieldResult<Fund> {
    let db_conn = root
        .db_connection_pool
        .get()
        .map_err(async_graphql::FieldError::from)?;
    tokio::task::spawn_blocking(move || {
        funds_table
            .filter(funds_dsl::id.eq(id))
            .first::<Fund>(&db_conn)
            .map_err(async_graphql::FieldError::from)
    })
    .await?
    .map_err(async_graphql::FieldError::from)
}
