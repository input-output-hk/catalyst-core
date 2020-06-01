use super::schema::QueryRoot;
use crate::db::{schema::proposals::dsl::proposals, Proposal};
use async_graphql::Context;
use diesel::RunQueryDsl;

#[async_graphql::Object]
impl QueryRoot {
    #[field(desc = "Proposal information")]
    async fn proposals<'ctx>(&self, _ctx: &Context<'_>) -> Vec<Proposal> {
        let db_conn = self
            .db_connection_pool
            .get()
            .expect("Error connecting to database");
        tokio::task::spawn_blocking(move || {
            proposals
                .load::<Proposal>(&db_conn)
                .expect("Error loading proposals")
        })
        .await
        .expect("Error loading proposals")
    }
}
