use super::logic;
use crate::v0::context::SharedContext;
use crate::v0::result::HandlerResult;
use warp::{Rejection, Reply};

pub async fn get_fund_by_id(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_fund_by_id(id, context).await))
}

pub async fn get_fund(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(HandlerResult(logic::get_fund(context).await))
}
