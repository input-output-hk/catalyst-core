use crate::db::Progress;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use crate::{db, BehindDuration};

pub async fn get_interval_behind_now(
    context: SharedContext,
) -> Result<BehindDuration, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    let result = db::query::behind(pool).await?;
    if result.is_empty() || result.len() > 1 {
        Err(HandleError::DatabaseInconsistency(
            "expected only 1 record for maximum block time".to_string(),
        ))
    } else {
        Ok(result[0].clone())
    }
}

pub async fn get_sync_progress(context: SharedContext) -> Result<Progress, HandleError> {
    let pool = &context.read().await.db_connection_pool;
    let result = db::query::sync_progress(pool).await?;

    if result.is_empty() || result.len() > 1 {
        Err(HandleError::DatabaseInconsistency(
            "expected only 1 record for sync progress".to_string(),
        ))
    } else {
        Ok(result[0].clone())
    }
}
