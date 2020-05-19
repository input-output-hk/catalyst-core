use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_id_from_hash(
    hash: String,
    context: SharedContext,
) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&"foo".to_string()))
}

pub async fn get_hash_from_id(id: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&"bar".to_string()))
}
