use super::logic;
use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_proposal(id: i32, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&logic::get_proposal(id, context).await))
}

pub async fn get_all_proposals(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&logic::get_all_proposals(context).await))
}
