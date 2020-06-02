use super::logic;
use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_fund(name: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&logic::get_fund(name, context).await))
}
