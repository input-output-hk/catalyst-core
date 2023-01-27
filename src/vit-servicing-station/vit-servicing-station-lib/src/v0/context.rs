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
    use diesel::Connection;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    use super::*;
    use crate::db;

    pub fn new_db_test_shared_context() -> SharedContext {
        let pool = db::load_db_connection_pool(&init_test_db()).unwrap();
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

    pub fn new_test_shared_context(block0_path: Vec<PathBuf>) -> SharedContext {
        let pool = db::load_db_connection_pool(&init_test_db()).unwrap();
        new_shared_context(pool, block0_path, "2.0")
    }

    fn init_test_db() -> String {
        let base_db_url = match std::env::var("TEST_DATABASE_URL") {
            Ok(url) => url,
            Err(std::env::VarError::NotPresent) => panic!("missing TEST_DATABASE_URL env var"),
            Err(e) => panic!("{}", e),
        };

        let name = thread_rng()
            .sample_iter(Alphanumeric)
            .filter(u8::is_ascii_alphabetic)
            .take(5)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();

        let conn = diesel::pg::PgConnection::establish(&base_db_url).unwrap();
        conn.execute(&format!("CREATE DATABASE {}", name)).unwrap();

        format!("{}/{}", base_db_url, name)
    }
}
