use super::logic;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn get_interval_behind_now_filter(
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_interval_behind_now(context).await))
}

pub async fn progress_filter(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_sync_progress(context).await))
}
