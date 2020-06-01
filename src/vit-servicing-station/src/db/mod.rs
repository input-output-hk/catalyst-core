pub mod models;
pub mod schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
pub use models::Proposal;

pub type DBConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn load_db_connection_pool(db_url: &str) -> Result<DBConnectionPool, Error> {
    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}
