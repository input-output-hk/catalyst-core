use crate::db::snapshot::SnapshotDb;
use axum::{extract::Path, routing::get, Router};
use std::sync::Arc;

pub fn snapshot<State: SnapshotDb + Send + Sync + 'static>(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/snapshot",
            get({
                let state = state.clone();
                move || versions_exec(state)
            }),
        )
        .route(
            "/snapshot/voter/:event/:voting_key",
            get({
                let state = state.clone();
                move |path| voter_exec(path, state)
            }),
        )
        .route(
            "/snapshot/delegator/:event/:stake_public_key",
            get(move |path| delegator_exec(path, state)),
        )
}

async fn versions_exec<State: SnapshotDb>(state: Arc<State>) -> String {
    tracing::debug!("");

    let snapshot_versions = state.get_snapshot_versions();
    serde_json::to_string(&snapshot_versions).unwrap()
}

async fn voter_exec<State: SnapshotDb>(
    Path((event, voting_key)): Path<(String, String)>,
    state: Arc<State>,
) -> String {
    tracing::debug!("event: {0}, voting_key: {1}", event, voting_key);

    let voter = state.get_voter(event, voting_key);
    serde_json::to_string(&voter).unwrap()
}

async fn delegator_exec<State: SnapshotDb>(
    Path((event, stake_public_key)): Path<(String, String)>,
    state: Arc<State>,
) -> String {
    tracing::debug!("event: {0}, stake_public_key: {1}", event, stake_public_key);

    let delegator = state.get_delegator(event, stake_public_key);
    serde_json::to_string(&delegator).unwrap()
}
