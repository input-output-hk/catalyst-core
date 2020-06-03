use crate::db;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub db_connection_pool: Arc<db::DBConnectionPool>,
}

impl Context {
    pub fn new(db_connection_pool: db::DBConnectionPool) -> Self {
        Self {
            db_connection_pool: Arc::new(db_connection_pool),
        }
    }
}

pub fn new_shared_context(db_connection_pool: db::DBConnectionPool) -> SharedContext {
    let context = Context::new(db_connection_pool);
    Arc::new(RwLock::new(context))
}
