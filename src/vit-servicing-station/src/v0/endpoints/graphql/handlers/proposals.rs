use crate::db::{
    models::proposals::Proposal,
    views_schema::full_proposals_info::dsl::full_proposals_info as proposals_table,
};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::Context;
use diesel::RunQueryDsl;

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
