use crate::db::{models::funds::Fund, schema::funds::dsl::funds as funds_table};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::{Context, FieldResult};
use diesel::RunQueryDsl;

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
