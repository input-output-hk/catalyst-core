pub mod migrations;
pub mod models;
pub mod queries;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type DbConnection = SqliteConnection;

pub type DbPoolConn = PooledConnection<ConnectionManager<DbConnection>>;
pub type DbConnectionPool = Pool<ConnectionManager<DbConnection>>;

// ⚠ WARNING ⚠ : This query is sqlite specific, would need to be changed if backend changes
const TEST_CONN_QUERY: &str = "
SELECT
    name
FROM
    sqlite_master
WHERE
    type ='table' AND
    name NOT LIKE 'sqlite_%';
";

pub fn load_db_connection_pool(db_url: &str) -> Result<DbConnectionPool, Error> {
    let manager = ConnectionManager::<DbConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;

    // test db connection or bubble up error
    let conn = pool.get()?;
    conn.execute(TEST_CONN_QUERY)?;

    Ok(pool)
}
