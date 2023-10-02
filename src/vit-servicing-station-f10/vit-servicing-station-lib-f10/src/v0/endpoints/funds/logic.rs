use crate::db::queries::funds::FundWithNext;
use crate::db::{models::funds::Fund, queries::funds as funds_queries};
use crate::v0::context::SharedContext;
use crate::v0::errors::HandleError;

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<Fund, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::query_fund_by_id(id, pool).await
}

pub async fn get_current_fund(context: SharedContext) -> Result<FundWithNext, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::query_current_fund(pool).await
}

pub async fn get_all_funds(context: SharedContext) -> Result<Vec<i32>, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::query_all_funds(pool).await
}

pub async fn put_fund(fund: Fund, context: SharedContext) -> Result<(), HandleError> {
    let pool = &context.read().await.db_connection_pool;
    funds_queries::put_fund(fund, pool).await
}
