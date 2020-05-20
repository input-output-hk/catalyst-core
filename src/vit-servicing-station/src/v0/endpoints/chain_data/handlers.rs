use super::logic;
use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_data_from_id(id: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    match logic::get_data_from_id(id, context).await {
        Some(data) => Ok(warp::reply::json(&data)),
        None => Err(warp::reject::not_found()),
    }
}
