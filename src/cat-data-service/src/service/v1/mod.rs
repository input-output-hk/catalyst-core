use crate::state::State;
use axum::Router;
use std::sync::Arc;

mod registration;

pub fn v1(state: Arc<State>) -> Router {
    let registration = registration::registration(state);

    Router::new().nest("/v1", registration)
}
