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
            get(proposals_by_voter_group_id_exec),
        )
        .route(
            "/proposal/:id/:voter_group_id",
            get(proposal_by_and_by_voter_group_id_exec),
        )
}

async fn proposals_exec(body: String) -> String {
    tracing::debug!("body: {0}", body);

    format!("body: {0}", body)
}

async fn proposals_by_voter_group_id_exec(Path(voter_group_id): Path<String>) -> String {
    tracing::debug!("voter group id: {0}", voter_group_id);

    format!("voter group id: {0}", voter_group_id)
}

async fn proposal_by_and_by_voter_group_id_exec(
    Path((id, voter_group_id)): Path<(String, String)>,
) -> String {
    tracing::debug!("id: {0}, voter group id: {1}", id, voter_group_id);

    format!("id: {0}, voter group id: {1}", id, voter_group_id)
}
