//! Catalyst Election Database crate
use bb8::{Pool, RunError};
use bb8_postgres::PostgresConnectionManager;
use dotenvy::dotenv;
use schema_check::SchemaVersion;
use std::env::{self, VarError};
use std::str::FromStr;
use tokio_postgres::NoTls;

mod config_table;
pub mod queries;
pub mod schema_check;
pub mod types;

/// Database URL Environment Variable name.
/// eg: "`postgres://catalyst-dev:CHANGE_ME@localhost/CatalystDev`"
const DATABASE_URL_ENVVAR: &str = "EVENT_DB_URL";

/// Database version this crate matches.
/// Must equal the last Migrations Version Number.
pub const DATABASE_SCHEMA_VERSION: i32 = 9;

/// Event database errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(" Schema in database does not match schema supported by the Crate. The current schema version: {was}, the schema version we expected: {expected}")]
    MismatchedSchema { was: i32, expected: i32 },
    #[error(transparent)]
    TokioPostgresError(#[from] tokio_postgres::Error),
    #[error(transparent)]
    TokioPostgresRunError(#[from] RunError<tokio_postgres::Error>),
    #[error(transparent)]
    VarError(#[from] VarError),
}

#[allow(unused)]
/// Connection to the Election Database
pub struct EventDB {
    // Internal database connection.  DO NOT MAKE PUBLIC.
    // All database operations (queries, inserts, etc) should be constrained
    // to this crate and should be exported with a clean data access api.
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

/// Establish a connection to the database, and check the schema is up-to-date.
///
/// # Parameters
///
/// * `url` set to the postgres connection string needed to connect to the
///   database.  IF it is None, then the env var "`DATABASE_URL`" will be used
///   for this connection string. eg:
///     "`postgres://catalyst-dev:CHANGE_ME@localhost/CatalystDev`"
///
/// # Errors
///
/// This function will return an error if:
/// * `url` is None and the environment variable "`DATABASE_URL`" isn't set.
/// * There is any error communicating the the database to check its schema.
/// * The database schema in the DB does not 100% match the schema supported by
///   this library.
///
/// # Notes
///
/// The env var "`DATABASE_URL`" can be set directly as an anv var, or in a
/// `.env` file.
pub async fn establish_connection(url: Option<&str>) -> Result<EventDB, Error> {
    // Support env vars in a `.env` file,  doesn't need to exist.
    dotenv().ok();

    let database_url = match url {
        Some(url) => url.to_string(),
        // If the Database connection URL is not supplied, try and get from the env var.
        None => env::var(DATABASE_URL_ENVVAR)?,
    };

    let config = tokio_postgres::config::Config::from_str(&database_url)?;

    let pg_mgr = PostgresConnectionManager::new(config, tokio_postgres::NoTls);

    let pool = Pool::builder().build(pg_mgr).await?;

    let db = EventDB { pool };

    db.schema_version_check().await?;

    Ok(db)
}

#[cfg(test)]
mod test {

    /// Check if the schema version in the DB is up to date.
    #[tokio::test]
    #[ignore = "not used"]
    async fn check_schema_version() {
        use crate::establish_connection;

        establish_connection(None).await.expect("pass");
    }
}
