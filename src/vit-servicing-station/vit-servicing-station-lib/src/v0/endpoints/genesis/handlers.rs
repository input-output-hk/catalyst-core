use crate::v0::context::SharedContext;
use warp::{http::Response, Rejection, Reply};

pub async fn get_genesis(context: SharedContext) -> Result<impl Reply, Rejection> {
    let mut response: Vec<u8> = context.read().await.block0.clone();

    // check if block0 is not loaded and try to load it again
    if response.is_empty() {
        let block0_path = context.read().await.block0_path.clone();
        response = tokio::fs::read(block0_path).await.unwrap_or_default();
        if !response.is_empty() {
            context.write().await.block0 = response.clone();
        }
    }

    // if we have no block0
    if response.is_empty() {
        Ok(Response::builder()
            .status(warp::http::status::StatusCode::NO_CONTENT)
            .header("Content-Type", "application/octet-stream")
            .body(response)
            .unwrap())
    // if we have a block0
    } else {
        Ok(Response::builder()
            .header("Content-Type", "application/octet-stream")
            .body(response)
            .unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::v0::context::test::new_test_shared_context;
    use warp::Filter;

    #[tokio::test]
    async fn get_block0_succeed() {
        // build context
        let block0_path = "../resources/tests/block0.bin";
        let shared_context = new_test_shared_context("", block0_path);
        let block0 = std::fs::read(block0_path).unwrap();

        let with_context = warp::any().map(move || shared_context.clone());

        // build filter
        let filter = warp::any()
            .and(warp::get())
            .and(with_context)
            .and_then(get_genesis);

        // check status code and block0 data
        let result = warp::test::request().method("GET").reply(&filter).await;

        assert_eq!(result.status(), warp::http::StatusCode::OK);
        let body = result.body().to_vec();
        assert_eq!(block0, body);
    }
}
