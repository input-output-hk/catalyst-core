use crate::db::{models::funds::Fund, schema::funds::dsl::funds as funds_table};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::Context;
use diesel::RunQueryDsl;

pub async fn funds<'ctx>(root: &QueryRoot, _ctx: &Context<'_>) -> Vec<Fund> {
    let db_conn = root
        .db_connection_pool
        .get()
        .expect("Error connecting to database");
    tokio::task::spawn_blocking(move || {
        funds_table
            .load::<Fund>(&db_conn)
            .expect("Error loading proposals")
    })
    .await
    .expect("Error loading proposals")
}
