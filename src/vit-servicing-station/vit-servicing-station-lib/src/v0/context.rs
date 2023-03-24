use crate::db;
use crate::v0::genesis_block::GenesisBlock;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SharedContext = Arc<RwLock<Context>>;

#[derive(Clone)]
pub struct Context {
    pub db_connection_pool: db::DbConnectionPool,
    pub block0: Vec<GenesisBlock>,
    pub versioning: String,
}

impl Context {
    pub fn new(
        db_connection_pool: db::DbConnectionPool,
        block0: Vec<GenesisBlock>,
        versioning: String,
    ) -> Self {
        Self {
            db_connection_pool,
            block0,
            versioning,
        }
    }
}

pub fn new_shared_context(
    db_connection_pool: db::DbConnectionPool,
    block0_path: Vec<PathBuf>,
    versioning: &str,
) -> SharedContext {
    let context = Context::new(
        db_connection_pool,
        block0_path
            .iter()
            .map(|x| GenesisBlock::from_str(x.to_str().unwrap()).unwrap())
            .collect(),
        versioning.to_string(),
    );
    Arc::new(RwLock::new(context))
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db;

    pub fn new_test_shared_context_from_url(db_url: &str) -> SharedContext {
        let pool = db::load_db_connection_pool(db_url).unwrap();
        let block0: Vec<u8> = vec![1, 2, 3, 4, 5];
        Arc::new(RwLock::new(Context::new(
            pool,
            vec![GenesisBlock {
                block0_path: "".to_string(),
                block0,
            }],
            "2.0".to_string(),
        )))
    }

    pub fn new_test_shared_context_with_block0(
        db_url: &str,
        block0_path: Vec<PathBuf>,
    ) -> SharedContext {
        let pool = db::load_db_connection_pool(&db_url).unwrap();
        new_shared_context(pool, block0_path, "2.0")
    }
}
