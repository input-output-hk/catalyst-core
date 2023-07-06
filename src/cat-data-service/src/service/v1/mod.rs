use crate::state::State;
use axum::Router;
use serde::Deserialize;
use std::sync::Arc;

mod event;
#[cfg(feature = "jorm-mock")]
mod jorm_mock;
mod registration;
mod search;

#[derive(Deserialize)]
struct LimitOffset {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn v1(state: Arc<State>) -> Router {
    #[cfg(feature = "jorm-mock")]
    let jorm_mock = jorm_mock::jorm_mock(state.clone());

    let registration = registration::registration(state.clone());
    let event = event::event(state.clone());
    let search = search::search(state);

    let router = registration.merge(event).merge(search);
    #[cfg(feature = "jorm-mock")]
    let router = router.merge(jorm_mock);

    Router::new().nest("/v1", router)
}
