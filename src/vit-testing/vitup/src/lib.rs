#[macro_use(error_chain, bail)]
extern crate error_chain;

pub mod error;
pub mod interactive;
mod scenario;
pub mod setup;

use error::Result;

use scenario::vit_station;
use scenario::wallet;
