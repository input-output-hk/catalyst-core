use super::logic;
use crate::rest::v0::context::SharedContext;
use crate::rest::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn submit_tx(data: Vec<u8>, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::submit_tx(data, context).await))
}
