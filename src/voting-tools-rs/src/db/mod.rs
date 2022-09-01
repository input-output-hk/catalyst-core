use color_eyre::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    result::QueryResult,
    PgConnection,
};
use microtype::secrecy::ExposeSecret;

use crate::config::DbConfig;

pub struct Db(Pool<ConnectionManager<PgConnection>>);

impl std::fmt::Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Db")
    }
}

mod queries;
#[allow(unused_imports)] // because custom types don't always appear in all tables, the import
// triggers this error
mod schema;
pub mod types;

type Conn = PooledConnection<ConnectionManager<PgConnection>>;

impl Db {
    pub async fn connect(
        DbConfig {
            name,
            user,
            host,
            password,
        }: DbConfig,
    ) -> Result<Self> {
        let url = format!(
            "postgres://{user}:{password}@{host}/{name}",
            password = password.expose_secret()
        );
        let manager = ConnectionManager::new(&url);
        let pool = Pool::new(manager)?;

        Ok(Db(pool))
    }

    fn exec<T, F: FnOnce(&Conn) -> QueryResult<T>>(&self, f: F) -> Result<T> {
        let conn = self.0.get()?;
        let result = f(&conn)?;
        Ok(result)
    }
}
