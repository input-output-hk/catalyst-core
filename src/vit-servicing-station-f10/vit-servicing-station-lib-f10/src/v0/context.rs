use crate::db;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub db_connection_pool: db::DbConnectionPool,
    pub block0_path: String,
    pub block0: Vec<u8>,
    pub versioning: String,
}

impl Context {
    pub fn new(
        db_connection_pool: db::DbConnectionPool,
        block0_path: &str,
        block0: Vec<u8>,
        versioning: String,
    ) -> Self {
        Self {
            db_connection_pool,
            block0_path: block0_path.to_string(),
            block0,
            versioning,
        }
    }
}

pub fn new_shared_context(
    db_connection_pool: db::DbConnectionPool,
    block0_path: &str,
    versioning: &str,
) -> SharedContext {
    let block0 = std::fs::read(block0_path).unwrap_or_default();
    let context = Context::new(
        db_connection_pool,
        block0_path,
        block0,
        versioning.to_string(),
    );
    Arc::new(RwLock::new(context))
}

#[cfg(test)]
pub mod test {
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    use super::*;
    use crate::db;

    pub fn new_in_memmory_db_test_shared_context() -> SharedContext {
        let name: String = thread_rng()
            .sample_iter(Alphanumeric)
            .take(5)
            .map(char::from)
            .collect();
        let db_url = format!("file:{}?mode=memory&cache=shared", name);
        let pool = db::load_db_connection_pool(&db_url).unwrap();
        let block0: Vec<u8> = vec![1, 2, 3, 4, 5];
        Arc::new(RwLock::new(Context::new(
            pool,
            "",
            block0,
            "2.0".to_string(),
        )))
    }

    pub fn new_test_shared_context(db_url: &str, block0_path: &str) -> SharedContext {
        let pool = db::load_db_connection_pool(db_url).unwrap();
        new_shared_context(pool, block0_path, "2.0")
    }
}
