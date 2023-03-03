use axum::Router;

mod chalenges;
mod fund;
mod snapshot;

pub fn v0() -> Router {
    let snapshot = snapshot::snapshot();
    let fund = fund::fund();
    let chalenges = chalenges::chalenges();

    Router::new().nest("/v0", snapshot.merge(fund).merge(chalenges))
}
