use crate::db::fund::Fund;
use axum::{extract::Path, routing::get, Router};

pub fn fund() -> Router {
    Router::new()
        .route("/funds", get(move || funds_exec()))
        .route("/fund", get(move || fund_exec()))
        .route("/fund/:id", get(move |path| fund_by_id_exec(path)))
}

async fn fund_exec() -> String {
    tracing::debug!("fund_exec");

    let current_fund: Fund = Default::default();
    serde_json::to_string(&current_fund).unwrap()
}

async fn fund_by_id_exec(Path(id): Path<i32>) -> String {
    tracing::debug!("fund_by_id_exec, id: {0}", id);

    let fund: Fund = Default::default();
    serde_json::to_string(&fund).unwrap()
}

async fn funds_exec() -> String {
    tracing::debug!("funds_exec");

    let fund_ids: Vec<i32> = Default::default();
    serde_json::to_string(&fund_ids).unwrap()
}
