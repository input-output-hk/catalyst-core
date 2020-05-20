use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_data_from_id(id: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    match context.read().await.static_chain_data.get(&id) {
        Some(data) => Ok(warp::reply::json(data)),
        None => Err(warp::reject::not_found()),
    }
}
