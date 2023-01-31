#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate clap;

pub mod db;
pub mod server;
pub mod utils;
pub mod v0;

#[cfg(test)]
mod testing;
