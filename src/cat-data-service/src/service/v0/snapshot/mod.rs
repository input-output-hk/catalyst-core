use self::{delegator::delegator, voter::voter};
use axum::Router;

mod delegator;
mod voter;

pub fn snapshot() -> Router {
    Router::new().nest("/snapshot", voter().merge(delegator()))
}
