use crate::db::{models::funds::Fund, queries::funds as funds_queries};
use crate::v0::endpoints::graphql::schema::QueryRoot;
use async_graphql::{Context, ErrorExtensions, FieldResult};

pub async fn funds<'ctx>(root: &QueryRoot, _ctx: &Context<'_>) -> FieldResult<Vec<Fund>> {
    funds_queries::query_all_funds(&root.db_connection_pool)
        .await
        .map_err(|e| e.extend())
}

pub async fn fund<'ctx>(root: &QueryRoot, fund_id: i32, _ctx: &Context<'_>) -> FieldResult<Fund> {
    funds_queries::query_fund_by_id(fund_id, &root.db_connection_pool)
        .await
        .map_err(|e| e.extend())
}
