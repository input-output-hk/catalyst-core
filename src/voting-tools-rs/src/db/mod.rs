use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use color_eyre::Result;
use microtype::secrecy::ExposeSecret;
use tokio_postgres::NoTls;

use crate::config::DbConfig;

#[derive(Debug)]
pub struct Db(Pool<PostgresConnectionManager<NoTls>>);

mod queries;

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

        let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(url, NoTls)?;
        let pool = bb8::Pool::builder().build(manager).await?;

        Ok(Db(pool))
    }

    async fn conn(&self) -> Result<PooledConnection<'_, PostgresConnectionManager<NoTls>>> {
        Ok(self.0.get().await?)
    }
}
