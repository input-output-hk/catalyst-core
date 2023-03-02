//! Catalyst Election Database crate

mod config_table;
mod schema_check;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use schema_check::SchemaVersion;
use tokio_postgres::NoTls;

use dotenvy::dotenv;

use std::env;
use std::error::Error;
use std::str::FromStr;

/// Database URL Environment Variable name.
/// eg: "`postgres://catalyst-dev:CHANGE_ME@localhost/CatalystDev`"
const DATABASE_URL_ENVVAR: &str = "ELECTION_DB_URL";

/// Database version this crate matches.
/// Must equal the last Migrations Version Number.
pub const DATABASE_SCHEMA_VERSION: u32 = 10;

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
///
/// # Examples
///
/// ```
/// let db = election_db::establish_connection(None)?;
/// ```
pub async fn establish_connection(
    url: Option<&str>,
) -> Result<EventDB, Box<dyn Error + Send + Sync + 'static>> {

    // Support env vars in a `.env` file,  doesn't need to exist.
    dotenv().ok();

    // If the Database connection URL is not supplied, try and get from the env var.
    let database_url = match url {
        Some(url) => url.to_string(),
        None => env::var(DATABASE_URL_ENVVAR)?,
    };

    let config = tokio_postgres::config::Config::from_str(&database_url)?;

    let pg_mgr = PostgresConnectionManager::new(
        config,
        tokio_postgres::NoTls,
    );

    let pool = Pool::builder().build(pg_mgr).await?;

    let db = EventDB { pool };

    db.schema_version_check().await?;

    Ok(db)
}

#[cfg(test)]
mod test {


    /// Check if the schema version in the DB is up to date.
    #[tokio::test]
    async fn check_schema_version() {
        use crate::establish_connection;

        establish_connection(None).await.expect("pass");
    }
}
