use axum::{extract::Path, routing::get, Router};

pub fn snapshot() -> Router {
    Router::new()
        .route("/snapshot", get(versions_exec))
        .route("/snapshot/voter/:event/:voting_key", get(voter_exec))
        .route(
            "/snapshot/delegator/:event/:stake_public_key",
            get(delegator_exec),
        )
}

async fn versions_exec() -> String {
    tracing::debug!("");

    "latest".to_string()
}

async fn voter_exec(Path((event, voting_key)): Path<(String, String)>) -> String {
    tracing::debug!("event: {0}, voting_key: {1}", event, voting_key);

    format!("voter, event: {0}, voting_ket: {1}", event, voting_key)
}

async fn delegator_exec(Path((event, stake_public_key)): Path<(String, String)>) -> String {
    tracing::debug!("event: {0}, stake_public_key: {1}", event, stake_public_key);

    format!(
        "delegator, event: {0}, stake_public_key: {1}",
        event, stake_public_key
    )
}
