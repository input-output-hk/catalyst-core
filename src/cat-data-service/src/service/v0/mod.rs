use axum::Router;

mod snapshot;

pub fn v0() -> Router {
    let snapshot = snapshot::snapshot();
    Router::new().nest("/v0", snapshot)
}
