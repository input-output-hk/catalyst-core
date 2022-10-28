pub mod migrations;
pub mod models;
pub mod queries;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
type Db = diesel::pg::Pg;
pub type DbConnection = PgConnection;

pub type DbPoolConn = PooledConnection<ConnectionManager<DbConnection>>;
pub type DbConnectionPool = Pool<ConnectionManager<DbConnection>>;

pub fn load_db_connection_pool(db_url: &str) -> Result<DbConnectionPool, Error> {
    let manager = ConnectionManager::<DbConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;

    Ok(pool)
}
