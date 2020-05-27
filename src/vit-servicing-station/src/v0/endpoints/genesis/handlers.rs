use crate::v0::context::SharedContext;
use warp::{Rejection, Reply};

// TODO: Implement for serving the genesis block for a given proposal id
pub async fn get_genesis_from_id(
    _id: String,
    _context: SharedContext,
) -> Result<impl Reply, Rejection> {
    let response: Vec<u8> = vec![];
    Ok(warp::reply::json(&response))
}
