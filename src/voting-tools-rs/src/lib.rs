//! Rust implementation of voting tools
//!
//! Original Haskell repository is <https://github.com/input-output-hk/voting-tools>
//!
//! The queries themselves (as well as the details of the CLI) are different, but they should
//! produce similar outputs. Malformed registrations are silently ignored.

#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq
)]

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate diesel;

mod cli;
mod db;
mod logic;
mod model;

#[cfg(test)]
mod testing;

// this export style forces us to be explicit about what is in the public API
pub use exports::*;
mod exports {
    pub use crate::cli::Args;
    pub use crate::db::{Conn, Db, DbConfig};
    pub use crate::logic::voting_power;
}
