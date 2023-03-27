use crate::state::State;
use axum::Router;
use event_db::queries::snapshot::SnapshotQueries;
use std::sync::Arc;

mod chalenges;
mod fund;
mod proposals;
mod reviews;
mod snapshot;

pub fn v0<EventDB: SnapshotQueries>(state: Arc<State<EventDB>>) -> Router {
    let snapshot = snapshot::snapshot(state.clone());
    let fund = fund::fund();
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
