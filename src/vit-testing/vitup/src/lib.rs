#![recursion_limit = "256"]
#![allow(clippy::result_large_err)]

pub mod builders;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod mode;
#[allow(clippy::all)]
pub mod testing;

pub type Result<T> = std::result::Result<T, error::Error>;
