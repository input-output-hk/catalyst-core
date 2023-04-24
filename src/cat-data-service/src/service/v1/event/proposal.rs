use crate::state::State;
use axum::Router;
use std::sync::Arc;

pub fn proposal(_state: Arc<State>) -> Router {
    Router::new()
}
