use axum::Router;

mod fund;
mod snapshot;

pub fn v0() -> Router {
    let snapshot = snapshot::snapshot();
    let fund = fund::fund();
    Router::new().nest("/v0", snapshot.merge(fund))
}
