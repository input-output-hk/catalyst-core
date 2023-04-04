use crate::{
    service::{handle_result, Error},
    state::State,
};
use axum::{extract::Path, routing::get, Router};
use event_db::types::snapshot::{Delegator, EventId, Voter};
use std::sync::Arc;

pub fn registration(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/registration/voter/:voting_key",
            get({
                let state = state.clone();
                move |path| async {
                    handle_result(voter_by_latest_event_exec(path, state).await).await
                }
            }),
        )
        .route(
            "/registration/voter/:voting_key/:event",
            get({
                let state = state.clone();
                move |path| async { handle_result(voter_by_event_exec(path, state).await).await }
            }),
        )
        .route(
            "/registration/delegations/:stake_public_key",
            get({
                let state = state.clone();
                move |path| async {
                    handle_result(delegations_by_latest_event_exec(path, state).await).await
                }
            }),
        )
        .route(
            "/registration/delegations/:stake_public_key/:event",
            get(move |path| async {
                handle_result(delegations_by_event_exec(path, state).await).await
            }),
        )
}

async fn voter_by_latest_event_exec(
    Path(voting_key): Path<String>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!("voter_by_latest_event_exec: voting_key: {0}", voting_key);

    let voter = state.event_db.get_voter(&None, voting_key).await?;
    Ok(voter)
}

async fn voter_by_event_exec(
    Path((voting_key, event)): Path<(String, EventId)>,
    state: Arc<State>,
) -> Result<Voter, Error> {
    tracing::debug!(
        "voter_by_event_exec: voting_key: {0}, event: {1}",
        voting_key,
        event.0
    );

    let voter = state.event_db.get_voter(&Some(event), voting_key).await?;
    Ok(voter)
}

async fn delegations_by_latest_event_exec(
    Path(stake_public_key): Path<String>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!("delegator_exec: stake_public_key: {0}", stake_public_key);

    let delegator = state
        .event_db
        .get_delegator(&None, stake_public_key)
        .await?;
    Ok(delegator)
}

async fn delegations_by_event_exec(
    Path((stake_public_key, event)): Path<(String, EventId)>,
    state: Arc<State>,
) -> Result<Delegator, Error> {
    tracing::debug!(
        "delegator_exec: stake_public_key: {0}, event: {1}",
        stake_public_key,
        event.0
    );

    let delegator = state
        .event_db
        .get_delegator(&Some(event), stake_public_key)
        .await?;
    Ok(delegator)
}
