use crate::{
    db::proposal::Proposal,
    service::{handle_result, Error},
};
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
            get(|path| async { handle_result(proposals_by_voter_group_id_exec(path).await).await }),
        )
        .route(
            "/proposal/:id/:voter_group_id",
            get(|path| async {
                handle_result(proposal_by_and_by_voter_group_id_exec(path).await).await
            }),
        )
}

async fn proposals_exec(body: String) -> String {
    tracing::debug!("proposals_exec, body: {0}", body);

    format!("body: {0}", body)
}

async fn proposals_by_voter_group_id_exec(
    Path(voter_group_id): Path<String>,
) -> Result<Vec<Proposal>, Error> {
    tracing::debug!(
        "proposals_by_voter_group_id_exec, voter group id: {0}",
        voter_group_id
    );

    let proposals: Vec<Proposal> = Default::default();
    Ok(proposals)
}

async fn proposal_by_and_by_voter_group_id_exec(
    Path((id, voter_group_id)): Path<(i32, String)>,
) -> Result<Proposal, Error> {
    tracing::debug!(
        "proposal_by_and_by_voter_group_id_exec, id: {0}, voter group id: {1}",
        id,
        voter_group_id
    );

    let proposal: Proposal = Default::default();
    Ok(proposal)
}
