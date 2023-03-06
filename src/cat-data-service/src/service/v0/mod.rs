use std::sync::Arc;

use axum::Router;

use crate::db::DB;

mod chalenges;
mod fund;
mod proposals;
mod reviews;
mod snapshot;

pub fn v0<State: DB + Send + Sync + 'static>(state: Arc<State>) -> Router {
    let snapshot = snapshot::snapshot(state.clone());
    let fund = fund::fund(state);
    let chalenges = chalenges::chalenges();
    let proposals = proposals::proposals();
    let reviews = reviews::reviews();

    Router::new().nest(
        "/v0",
        snapshot
            .merge(fund)
            .merge(chalenges)
            .merge(proposals)
            .merge(reviews),
    )
}
