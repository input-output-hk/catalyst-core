use crate::state::State;
use axum::{extract::Path, routing::get, Router};
use event_db::queries::snapshot::SnapshotQueries;
use std::sync::Arc;

pub fn snapshot<EventDB: SnapshotQueries>(state: Arc<State<EventDB>>) -> Router {
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

async fn versions_exec<EventDB: SnapshotQueries>(state: Arc<State<EventDB>>) -> String {
    tracing::debug!("versions_exec");

    let snapshot_versions = state.event_db.get_snapshot_versions().await.unwrap();
    serde_json::to_string(&snapshot_versions).unwrap()
}

async fn voter_exec<EventDB: SnapshotQueries>(
    Path((event, voting_key)): Path<(String, String)>,
    state: Arc<State<EventDB>>,
) -> String {
    tracing::debug!("voter_exec, event: {0}, voting_key: {1}", event, voting_key);

    let voter = state.event_db.get_voter(event, voting_key).await.unwrap();
    serde_json::to_string(&voter).unwrap()
}

async fn delegator_exec<EventDB: SnapshotQueries>(
    Path((event, stake_public_key)): Path<(String, String)>,
    state: Arc<State<EventDB>>,
) -> String {
    tracing::debug!(
        "delegator_exec, event: {0}, stake_public_key: {1}",
        event,
        stake_public_key
    );

    let delegator = state
        .event_db
        .get_delegator(event, stake_public_key)
        .await
        .unwrap();
    serde_json::to_string(&delegator).unwrap()
}
