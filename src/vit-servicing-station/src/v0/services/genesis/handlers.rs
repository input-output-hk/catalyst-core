use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_genesis_from_id(
    id: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let response: Vec<u8> = vec![];
    Ok(warp::reply::json(&response))
}
