use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use event_db::types::snapshot::{Delegator, SnapshotVersion, Voter};
use std::sync::Arc;

pub fn snapshot(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/snapshot",
            get({
                let state = state.clone();
                move || async { handle_result(versions_exec(state).await).await }
            }),
        )
        .route(
            "/snapshot/voter/:event/:voting_key",
            get({
                let state = state.clone();
                move |path| async { handle_result(voter_exec(path, state).await).await }
            }),
        )
        .route(
            "/snapshot/delegator/:event/:stake_public_key",
            get(move |path| async { handle_result(delegator_exec(path, state).await).await }),
        )
}

async fn versions_exec(state: Arc<State>) -> Result<Vec<SnapshotVersion>, Error> {
    tracing::debug!("versions_exec");

    let snapshot_versions = state.event_db.get_snapshot_versions().await?;
    Ok(snapshot_versions)
}

async fn voter_exec(
    Path((event, voting_key)): Path<(String, String)>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!("voter_exec, event: {0}, voting_key: {1}", event, voting_key);

    let voter = state.event_db.get_voter(event, voting_key).await?;
    Ok(voter)
}

async fn delegator_exec(
    Path((event, stake_public_key)): Path<(String, String)>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!(
        "delegator_exec, event: {0}, stake_public_key: {1}",
        event,
        stake_public_key
    );

    let delegator = state
        .event_db
        .get_delegator(event, stake_public_key)
        .await?;
    Ok(delegator)
}
