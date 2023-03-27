use crate::{
    db::snapshot::{Delegator, Voter},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use std::sync::Arc;

pub fn snapshot(state: Arc<State>) -> Router {
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

async fn versions_exec(_state: Arc<State>) -> String {
    tracing::debug!("versions_exec");

    let snapshot_versions: Vec<String> = Default::default();
    serde_json::to_string(&snapshot_versions).unwrap()
}

async fn voter_exec(
    Path((event, voting_key)): Path<(String, String)>,
    _state: Arc<State>,
) -> String {
    tracing::debug!("voter_exec, event: {0}, voting_key: {1}", event, voting_key);

    let voter: Voter = Default::default();
    serde_json::to_string(&voter).unwrap()
}

async fn delegator_exec(
    Path((event, stake_public_key)): Path<(String, String)>,
    _state: Arc<State>,
) -> String {
    tracing::debug!(
        "delegator_exec, event: {0}, stake_public_key: {1}",
        event,
        stake_public_key
    );

    let delegator: Delegator = Default::default();
    serde_json::to_string(&delegator).unwrap()
}
