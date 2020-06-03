use super::logic;
use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_fund(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&logic::get_fund(id, context).await))
}
