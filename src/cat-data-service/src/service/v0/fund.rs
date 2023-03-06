use crate::db::fund::FundDb;
use axum::{extract::Path, routing::get, Router};
use std::sync::Arc;

pub fn fund<State: FundDb + Send + Sync + 'static>(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/funds",
            get({
                let state = state.clone();
                move || funds_exec(state)
            }),
        )
        .route(
            "/fund",
            get({
                let state = state.clone();
                move || fund_exec(state)
            }),
        )
        .route("/fund/:id", get(move |path| fund_by_id_exec(path, state)))
}

async fn fund_exec<State: FundDb>(state: Arc<State>) -> String {
    tracing::debug!("");

    let current_fund = state.get_current_fund();
    serde_json::to_string(&current_fund).unwrap()
}

async fn fund_by_id_exec<State: FundDb>(Path(id): Path<i32>, state: Arc<State>) -> String {
    tracing::debug!("id: {0}", id);

    let fund = state.get_fund_by_id(id);
    serde_json::to_string(&fund).unwrap()
}

async fn funds_exec<State: FundDb>(state: Arc<State>) -> String {
    tracing::debug!("");

    let fund_ids = state.get_fund_ids();
    serde_json::to_string(&fund_ids).unwrap()
}
