pub mod models;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;

pub type DBConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
// TODO: Right now this is force as the current backend. But it should be abstracted so it works for any diesel::Backend
type DB = diesel::sqlite::Sqlite;

pub fn load_db_connection_pool(db_url: &str) -> Result<DBConnectionPool, Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}
