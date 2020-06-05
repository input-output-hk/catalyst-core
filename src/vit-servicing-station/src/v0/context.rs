use crate::db;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub db_connection_pool: db::DBConnectionPool,
    pub block0: Vec<u8>,
}

impl Context {
    pub fn new(db_connection_pool: db::DBConnectionPool, block0_path: &str) -> Self {
        Self {
            db_connection_pool,
            block0: load_block0(block0_path),
        }
    }
}

pub fn new_shared_context(
    db_connection_pool: db::DBConnectionPool,
    block0_path: &str,
) -> SharedContext {
    let context = Context::new(db_connection_pool, block0_path);
    Arc::new(RwLock::new(context))
}

pub fn load_block0(block0_path: &str) -> Vec<u8> {
    std::fs::read(block0_path).unwrap()
}
