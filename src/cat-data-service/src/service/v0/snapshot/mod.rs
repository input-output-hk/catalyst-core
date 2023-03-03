use self::{delegator::delegator, voter::voter};
use axum::{routing::get, Router};

mod delegator;
mod voter;

pub fn snapshot() -> Router {
    let root = Router::new().route("/", get(versions_exec));

    Router::new().nest("/snapshot", root.merge(voter().merge(delegator())))
}

async fn versions_exec() -> String {
    tracing::debug!("");

    "latest".to_string()
}
