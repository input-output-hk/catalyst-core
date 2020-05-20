use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

pub async fn get_data_from_id(id: String, context: SharedContext) -> Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&"bar".to_string()))
}
