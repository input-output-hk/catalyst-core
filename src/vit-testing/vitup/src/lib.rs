#[macro_use(error_chain, bail)]
extern crate error_chain;

pub mod config;
pub mod error;
pub mod interactive;
pub mod manager;
pub mod scenario;
pub mod setup;

use error::Result;

use scenario::vit_station;
use scenario::wallet;
