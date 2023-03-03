use axum::{extract::Path, routing::get, Router};

pub fn chalenges() -> Router {
    Router::new()
        .route("/chalenges", get(chalenges_exec))
        .route("/chalenges/:id", get(chalenge_by_id_exec))
        .route(
            "/chalenges/:id/:voter_group_id",
            get(chalenge_by_id_and_voter_group_id_exec),
        )
}

async fn chalenges_exec() -> String {
    tracing::debug!("");

    "latest".to_string()
}

async fn chalenge_by_id_exec(Path(id): Path<String>) -> String {
    tracing::debug!("id: {0}", id);

    format!("chalenge id: {0}", id)
}

async fn chalenge_by_id_and_voter_group_id_exec(
    Path((id, voter_group_id)): Path<(String, String)>,
) -> String {
    tracing::debug!("id: {0}, voter group id: {1}", id, voter_group_id);

    format!("chalenge id: {0}, voter group id: {1}", id, voter_group_id)
}
