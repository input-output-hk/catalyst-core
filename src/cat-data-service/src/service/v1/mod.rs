use crate::state::State;
use axum::Router;
use serde::Deserialize;
use std::sync::Arc;

mod event;
mod registration;
mod search;

#[derive(Deserialize)]
struct LimitOffset {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub fn v1(state: Arc<State>) -> Router {
    let registration = registration::registration(state.clone());
    let event = event::event(state.clone());
    let search = search::search(state);

    Router::new().nest("/v1", registration.merge(event).merge(search))
}
