use crate::state::State;
use axum::Router;
use std::sync::Arc;

mod chalenges;
mod fund;
mod proposals;
mod registration;
mod reviews;

pub fn v1(state: Arc<State>) -> Router {
    let registration = registration::registration(state);
    let fund = fund::fund();
    let chalenges = chalenges::chalenges();
    let proposals = proposals::proposals();
    let reviews = reviews::reviews();

    Router::new().nest(
        "/v1",
        registration
            .merge(fund)
            .merge(chalenges)
            .merge(proposals)
            .merge(reviews),
    )
}
