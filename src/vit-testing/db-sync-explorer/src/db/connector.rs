use crate::db::config::DbConfig;
use color_eyre::Result;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use microtype::secrecy::ExposeSecret;
use microtype::secrecy::Zeroize;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
/// Connect to the database with the provided credentials
///
/// # Errors
///
/// Returns an error if connecting to the database fails
pub fn connect(
    DbConfig {
        name,
        user,
        host,
        password,
    }: DbConfig,
) -> Result<DbPool> {
    let mut password = password
        .map(|p| format!(":{}", p.expose_secret()))
        .unwrap_or_default();

    let url = format!("postgres://{user}{password}@{host}/{name}");
    let pool = Pool::builder().build(ConnectionManager::new(url))?;
    password.zeroize();
    Ok(pool)
}
