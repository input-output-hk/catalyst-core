use axum::{extract::Path, routing::get, Router};

pub fn reviews() -> Router {
    Router::new().route("/reviews/:proposal_id", get(reviews_by_proposal_id_id_exec))
}

async fn reviews_by_proposal_id_id_exec(Path(proposal_id): Path<String>) -> String {
    tracing::debug!("reviews_by_proposal_id_id_exec, proposal id: {0}", proposal_id);

    format!("proposal id: {0}", proposal_id)
}
