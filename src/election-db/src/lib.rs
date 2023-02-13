//! Catalyst Election Database crate
#![recursion_limit = "256"]

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_autoincrement_new_struct;

mod config_table;
mod models;
mod schema;
mod schema_check;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use schema_check::db_version_check;
use std::env;
use std::error::Error;

/// Database URL Environment Variable name.
/// eg: "`postgres://catalyst-dev:CHANGE_ME@localhost/CatalystDev`"
const DATABASE_URL_ENVVAR: &str = "ELECTION_DB_URL";

#[allow(unused)]
/// Connection to the Election Database
pub struct ElectionDB {
    /// Internal database connection.  DO NOT MAKE PUBLIC.
    /// All database operations (queries, inserts, etc) should be constrained
    /// to this crate and should be exported with a clean data access api.
    conn: PgConnection,
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
pub fn establish_connection(
    url: Option<&str>,
) -> Result<ElectionDB, Box<dyn Error + Send + Sync + 'static>> {
    // Support env vars in a `.env` file,  doesn't need to exist.
    dotenv().ok();

    // If the Database connection URL is not supplied, try and get from the env var.
    let database_url = match url {
        Some(url) => url.to_string(),
        None => env::var(DATABASE_URL_ENVVAR)?,
    };

    let mut conn = PgConnection::establish(database_url.as_str())?;
    db_version_check(&mut conn)?;

    let db = ElectionDB { conn };

    Ok(db)
}

#[cfg(test)]
mod test {
    /// Check if the schema version in the DB is up to date.
    #[test]
    fn check_schema_version() {
        use crate::establish_connection;

        establish_connection(None).unwrap();
    }
}
