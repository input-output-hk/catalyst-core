use axum::{extract::Path, routing::get, Router};

pub fn voter() -> Router {
    Router::new().route("/voter/:event/:voting_key", get(voter_exec))
}

async fn voter_exec(Path((event, voting_key)): Path<(String, String)>) -> String {
    tracing::debug!("event: {0}, voting_key: {1}", event, voting_key);

    format!("voter, event: {0}, voting_ket: {1}", event, voting_key)
}
