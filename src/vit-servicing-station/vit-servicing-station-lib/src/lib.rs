#![allow(opaque_hidden_inferred_bound)]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate structopt;

pub mod db;
pub mod server;
pub mod utils;
pub mod v0;

#[cfg(test)]
mod testing;
