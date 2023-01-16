use super::logic;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn get_db_sync_content(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_db_sync_content(context).await))
}
