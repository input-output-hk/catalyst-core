use super::logic;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn get_meta_info(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_meta_info(context).await))
}
