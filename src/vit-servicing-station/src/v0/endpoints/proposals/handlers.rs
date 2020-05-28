use super::logic;
use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_data_from_id(id: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    match logic::get_data_from_id(id, context).await {
        Some(data) => Ok(warp::reply::json(&data)),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn get_all_proposals(context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&logic::get_all_proposals(context).await))
}
