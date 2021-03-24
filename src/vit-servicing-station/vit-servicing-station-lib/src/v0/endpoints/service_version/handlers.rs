use super::logic;
use crate::v0::{context::SharedContext, result::HandlerResult};
use warp::{Rejection, Reply};

pub async fn service_version(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::service_version(context).await))
}
