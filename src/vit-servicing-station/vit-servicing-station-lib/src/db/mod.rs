pub mod migrations;
pub mod models;
pub mod queries;
pub mod schema;
pub mod views_schema;

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use r2d2::PooledConnection;

#[derive(thiserror::Error, Debug)]
#[error("{0}")]
pub struct Error(r2d2::Error);

pub type DbConnectionPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = PooledConnection<ConnectionManager<PgConnection>>;

pub fn load_db_connection_pool(db_url: &str) -> Result<DbConnectionPool, Error> {
    Pool::builder()
        .build(ConnectionManager::<PgConnection>::new(db_url))
        .map_err(Error)
}
