use crate::db::proposal::Proposal;
use axum::{
    extract::Path,
    routing::{get, post},
    Router,
};

pub fn proposals() -> Router {
    Router::new()
        .route("/proposals", post(proposals_exec))
        .route(
            "/proposals/:voter_group_id",
            get(move |path| proposals_by_voter_group_id_exec(path)),
        )
        .route(
            "/proposal/:id/:voter_group_id",
            get(move |path| proposal_by_and_by_voter_group_id_exec(path)),
        )
}

async fn proposals_exec(body: String) -> String {
    tracing::debug!("proposals_exec, body: {0}", body);

    format!("body: {0}", body)
}

async fn proposals_by_voter_group_id_exec(Path(voter_group_id): Path<String>) -> String {
    tracing::debug!(
        "proposals_by_voter_group_id_exec, voter group id: {0}",
        voter_group_id
    );

    let proposals: Vec<Proposal> = Default::default();
    serde_json::to_string(&proposals).unwrap()
}

async fn proposal_by_and_by_voter_group_id_exec(
    Path((id, voter_group_id)): Path<(i32, String)>,
) -> String {
    tracing::debug!(
        "proposal_by_and_by_voter_group_id_exec, id: {0}, voter group id: {1}",
        id,
        voter_group_id
    );

    let proposal: Proposal = Default::default();
    serde_json::to_string(&proposal).unwrap()
}
