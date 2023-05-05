use crate::state::State;
use axum::Router;
use std::sync::Arc;

mod event;
mod registration;
mod search;

pub fn v1(state: Arc<State>) -> Router {
    let registration = registration::registration(state.clone());
    let event = event::event(state.clone());
    let search = search::search(state);

    Router::new().nest("/v1", registration.merge(event).merge(search))
}
