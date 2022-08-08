use color_eyre::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
use microtype::secrecy::ExposeSecret;

use crate::config::DbConfig;

pub struct Db(Pool<ConnectionManager<PgConnection>>);

type PoolConn = PooledConnection<ConnectionManager<PgConnection>>;

impl Db {
    pub fn connect(
        DbConfig {
            name,
            user,
            host,
            password,
        }: &DbConfig,
    ) -> Result<Self> {
        let url = format!(
            "postgres://{user}:{password}@{host}/{name}",
            password = password.expose_secret()
        );

        let manager = ConnectionManager::new(&url);
        let pool = Pool::builder().build(manager)?;

        Ok(Db(pool))
    }

    async fn exec<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&PoolConn) -> diesel::result::QueryResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let conn = self.0.get()?;
        let result = tokio::task::spawn_blocking(move || f(&conn)).await?;
        let value = result?;

        Ok(value)
    }
}
