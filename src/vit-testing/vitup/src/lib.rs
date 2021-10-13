#![recursion_limit = "256"]
#[macro_use(error_chain, bail)]
extern crate error_chain;

pub mod builders;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod interactive;
pub mod manager;
mod mock;
pub mod scenario;

use error::Result;

use scenario::vit_station;
use scenario::wallet;
