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
    use diesel::{
        r2d2::{ConnectionManager, Pool},
        Connection, RunQueryDsl,
    };
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    use super::*;
    use crate::db::DbConnection;

    const TEST_DATABASE_URL_ENV_KEY: &str = "TEST_DATABASE_URL";

    pub fn new_default_test_shared_context() -> SharedContext {
        let block0: Vec<u8> = vec![1, 2, 3, 4, 5];

        Arc::new(RwLock::new(Context::new(
            new_db_pool(),
            vec![GenesisBlock {
                block0_path: "".to_string(),
                block0,
            }],
            "2.0".to_string(),
        )))
    }

    pub fn new_test_shared_context(block0_path: Vec<PathBuf>) -> SharedContext {
        new_shared_context(new_db_pool(), block0_path, "2.0")
    }

    fn new_db_pool() -> Pool<ConnectionManager<DbConnection>> {
        let db_url =
            std::env::var(TEST_DATABASE_URL_ENV_KEY).expect("Missing test database URL env var");

        let db_name = {
            let conn = diesel::PgConnection::establish(&db_url).unwrap();

            let db_name: String = thread_rng()
                .sample_iter(Alphanumeric)
                .filter(u8::is_ascii_alphabetic)
                .take(16)
                .map(|c| char::from(c.to_ascii_lowercase()))
                .collect();
            let db_name = format!("test_{}", db_name);

            println!("Creating test database: {}", db_name);
            diesel::sql_query(format!("CREATE DATABASE {}", db_name))
                .execute(&conn)
                .unwrap();

            db_name
        };

        let manager = ConnectionManager::<DbConnection>::new(format!(
            "postgres://postgres:123456@localhost/{}",
            db_name
        ));

        Pool::builder().max_size(3).build(manager).unwrap()
    }
}
