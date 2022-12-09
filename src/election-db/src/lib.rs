//! Catalyst Election Database crate
#![recursion_limit = "256"]

#[macro_use] 
extern crate diesel;

#[macro_use] 
extern crate diesel_autoincrement_new_struct;

pub mod models;
pub mod schema;
mod schema_check;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::error::Error;

///  Establish a connection to the database, and check the schema is up-to-date.
#[must_use] 
pub fn establish_connection(url : Option<&str>) -> Result<PgConnection, Box<dyn Error + Send + Sync + 'static>> {
    // Support env vars in a `.env` file
    dotenv().ok();

    // If the Database connection URL is not supplied, try and get from the env var.
    let database_url = match url {
        Some(url) => url,
        None => &env::var("DATABASE_URL")?
    };

    let con = PgConnection::establish(database_url)?;

    Ok(con)
}