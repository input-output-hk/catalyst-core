pub mod migrations;
pub mod models;
pub mod queries;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use diesel::Connection;

pub type DBConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
// TODO: Right now this is forced as the current backend. But it should be abstracted so it works for any diesel::Backend
type DB = diesel::sqlite::Sqlite;
pub type DBConnection = SqliteConnection;

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

pub fn load_db_connection_pool(db_url: &str) -> Result<DBConnectionPool, Error> {
    // check if db file exists
    if !std::path::Path::new(db_url).exists() {
        return Err(Box::new(diesel::ConnectionError::InvalidConnectionUrl(
            format!("{} url does not exists", db_url.to_string()),
        )));
    }
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;

    // test db connection or bubble up error
    let conn = pool.get()?;
    conn.execute(TEST_CONN_QUERY)?;

    Ok(pool)
}
