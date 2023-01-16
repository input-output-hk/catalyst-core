use crate::db::Progress;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::errors::HandleError;
use crate::BehindDuration;

pub async fn get_interval_behind_now(
    context: SharedContext,
) -> Result<BehindDuration, HandleError> {
    let context = context.read().await;
    context.provider().get_interval_behind_now().await
}

pub async fn get_sync_progress(context: SharedContext) -> Result<Progress, HandleError> {
    let context = context.read().await;
    context.provider().get_sync_progress().await
}
