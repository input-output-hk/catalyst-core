use crate::v0::context::SharedContext;
use warp::{http::Response, Rejection, Reply};

pub async fn get_genesis(context: SharedContext) -> Result<impl Reply, Rejection> {
    let response: Vec<u8> = context.read().await.block0.clone();
    Ok(Response::builder()
        .header("Content-Type", "arraybuffer/blob")
        .body(response)
        .unwrap())
}
