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

///  Establish a connection to the database, and check the schema is up-to-date.
pub fn establish_connection(url : Option<&str>) -> PgConnection {
    // Support env vars in a `.env` file
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}