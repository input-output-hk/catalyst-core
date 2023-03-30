use crate::{
    db::fund::Fund,
    service::{handle_result, Error},
};
use axum::{extract::Path, routing::get, Router};

pub fn fund() -> Router {
    Router::new()
        .route(
            "/funds",
            get(|| async { handle_result(funds_exec().await).await }),
        )
        .route(
            "/fund",
            get(|| async { handle_result(fund_exec().await).await }),
        )
        .route(
            "/fund/:id",
            get(|path| async { handle_result(fund_by_id_exec(path).await).await }),
        )
}

async fn fund_exec() -> Result<Fund, Error> {
    tracing::debug!("fund_exec");

    let current_fund = Fund::default();
    Ok(current_fund)
}

async fn fund_by_id_exec(Path(id): Path<i32>) -> Result<Fund, Error> {
    tracing::debug!("fund_by_id_exec, id: {0}", id);

    let fund = Fund::default();
    Ok(fund)
}

async fn funds_exec() -> Result<Vec<i32>, Error> {
    tracing::debug!("funds_exec");

    let fund_ids: Vec<i32> = Default::default();
    Ok(fund_ids)
}
