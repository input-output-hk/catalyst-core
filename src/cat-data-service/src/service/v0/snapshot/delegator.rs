use axum::{extract::Path, routing::get, Router};

pub fn delegator() -> Router {
    Router::new().route("/delegator/:event/:stake_public_key", get(delegator_exec))
}

async fn delegator_exec(Path((event, stake_public_key)): Path<(String, String)>) -> String {
    format!(
        "delegator, event: {0}, stake_public_key: {1}",
        event, stake_public_key
    )
}
