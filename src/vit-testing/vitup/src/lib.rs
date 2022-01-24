#![recursion_limit = "256"]

pub mod builders;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod interactive;
pub mod manager;
mod mock;
pub mod scenario;

pub type Result<T> = std::result::Result<T, error::Error>;

use scenario::vit_station;
use scenario::wallet;
