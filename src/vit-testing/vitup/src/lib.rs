#![recursion_limit = "256"]
#[macro_use(error_chain, bail)]
extern crate error_chain;

pub mod client;
pub mod config;
pub mod error;
pub mod interactive;
pub mod manager;
mod mock;
pub mod scenario;
pub mod setup;

use error::Result;

use scenario::vit_station;
use scenario::wallet;
