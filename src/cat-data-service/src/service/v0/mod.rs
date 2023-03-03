use std::sync::Arc;

use axum::Router;

use crate::db::DB;

mod chalenges;
mod fund;
mod snapshot;

pub fn v0<State: DB + Send + Sync + 'static>(state: Arc<State>) -> Router {
    let snapshot = snapshot::snapshot(state);
    let fund = fund::fund();
    let chalenges = chalenges::chalenges();

    Router::new().nest("/v0", snapshot.merge(fund).merge(chalenges))
}
