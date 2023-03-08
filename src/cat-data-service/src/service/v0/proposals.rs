use crate::db::proposal::ProposalDb;
use axum::{
    extract::Path,
    routing::{get, post},
    Router,
};
use std::sync::Arc;

pub fn proposals<State: ProposalDb + Send + Sync + 'static>(state: Arc<State>) -> Router {
    Router::new()
        .route("/proposals", post(proposals_exec))
        .route(
            "/proposals/:voter_group_id",
            get({
                let state = state.clone();
                move |path| proposals_by_voter_group_id_exec(path, state)
            }),
        )
        .route(
            "/proposal/:id/:voter_group_id",
            get(move |path| proposal_by_and_by_voter_group_id_exec(path, state)),
        )
}

async fn proposals_exec(body: String) -> String {
    tracing::debug!("proposals_exec, body: {0}", body);

    format!("body: {0}", body)
}

async fn proposals_by_voter_group_id_exec<State: ProposalDb>(
    Path(voter_group_id): Path<String>,
    state: Arc<State>,
) -> String {
    tracing::debug!("proposals_by_voter_group_id_exec, voter group id: {0}", voter_group_id);

    let proposals = state.get_proposals_by_voter_group_id(voter_group_id);
    serde_json::to_string(&proposals).unwrap()
}

async fn proposal_by_and_by_voter_group_id_exec<State: ProposalDb>(
    Path((id, voter_group_id)): Path<(i32, String)>,
    state: Arc<State>,
) -> String {
    tracing::debug!("proposal_by_and_by_voter_group_id_exec, id: {0}, voter group id: {1}", id, voter_group_id);

    let proposal = state.get_proposal_by_and_by_voter_group_id(id, voter_group_id);
    serde_json::to_string(&proposal).unwrap()
}
