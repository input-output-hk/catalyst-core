#![recursion_limit = "256"]

pub mod builders;
pub mod cli;
pub mod client;
pub mod config;
pub mod error;
pub mod mode;
pub mod testing;

pub type Result<T> = std::result::Result<T, error::Error>;
