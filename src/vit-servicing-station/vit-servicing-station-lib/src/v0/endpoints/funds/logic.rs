use crate::db::{models::funds::Fund, queries::funds as funds_queries};
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<Fund, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::query_fund_by_id(id, pool).await
}

pub async fn get_fund(context: SharedContext) -> Result<Fund, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::query_fund(pool).await
}

