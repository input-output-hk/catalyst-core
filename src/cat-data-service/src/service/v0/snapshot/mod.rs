use axum::{routing::get, Router};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn snapshot() -> Router {
    Router::new()
        .route("/snapshot/tag", get(tag))
        .route("/snapshot/time", get(time))
}

async fn tag() -> String {
    "fund 9".to_string()
}

async fn time() -> String {
    format!(
        "time: {}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    )
}
