use axum::{extract::Path, routing::get, Router};

pub fn fund() -> Router {
    let funds = Router::new()
        .route("/funds", get(funds_exec))
        .route("/fund", get(fund_exec))
        .route("/fund/:id", get(fund_by_id_exec));

    funds
}

async fn fund_exec() -> String {
    tracing::debug!("");

    "latest fund".to_string()
}

async fn fund_by_id_exec(Path(id): Path<String>) -> String {
    tracing::debug!("id: {0}", id);

    format!("fund id: {0}", id)
}

async fn funds_exec() -> String {
    tracing::debug!("");

    "latest".to_string()
}
