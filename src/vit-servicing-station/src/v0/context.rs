use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {}

pub fn new_default_context() -> SharedContext {
    Arc::new(RwLock::new(Context {}))
}
