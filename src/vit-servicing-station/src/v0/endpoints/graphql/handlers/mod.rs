pub mod funds;
pub mod proposals;

use crate::db::{models::funds::Fund, models::proposals::Proposal};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::Context;

#[async_graphql::Object]
impl QueryRoot {
    #[field(desc = "Proposal information")]
    async fn proposals<'ctx>(&self, _ctx: &Context<'_>) -> Vec<Proposal> {
        proposals::proposals(&self, _ctx).await
    }

    #[field(desc = "Funds information")]
    async fn funds<'ctx>(&self, _ctx: &Context<'_>) -> Vec<Fund> {
        funds::funds(&self, _ctx).await
    }
}
