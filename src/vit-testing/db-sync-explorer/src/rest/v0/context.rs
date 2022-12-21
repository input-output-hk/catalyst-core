use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::DbPool;

pub type SharedContext = Arc<RwLock<Context>>;

pub struct Context {
    pub db_connection_pool: DbPool,
    pub token: Option<String>,
}

impl Context {
    pub fn new(db_connection_pool: DbPool, token: Option<String>) -> Self {
        Self {
            db_connection_pool,
            token,
        }
    }

    pub fn token(&self) -> &Option<String> {
        &self.token
    }
}

pub fn new_shared_context(db_connection_pool: DbPool, token: Option<String>) -> SharedContext {
    let context = Context::new(db_connection_pool, token);
    Arc::new(RwLock::new(context))
}
