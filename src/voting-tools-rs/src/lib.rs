//! Rust implementation of voting tools
//!
//! Original Haskell repository is <https://github.com/input-output-hk/voting-tools>
//!
//! The queries themselves (as well as the details of the CLI) are different, but they should
//! produce similar outputs. Malformed registrations are silently ignored.

#![warn(clippy::pedantic)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(clippy::integer_arithmetic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq,
    clippy::missing_panics_doc
)]

#[macro_use]
extern crate tracing;
#[macro_use]
extern crate diesel;

mod cli;
mod data;
mod data_provider;
mod db;
mod error;
// mod logic;
// mod model;
mod logic_2;
mod testing;
mod validation;

// this export style forces us to be explicit about what is in the public API
pub use exports::*;
mod exports {
    pub use crate::cli::{show_error_warning, Args, DryRunCommand};
    pub use crate::data::{
        Sig, Signature, SlotNo, SnapshotEntry, VotingPowerSource, VotingPurpose,
    };
    pub use crate::data_provider::DataProvider;
    pub use crate::db::{Conn, Db, DbConfig};
    pub use crate::logic_2::{voting_power, VotingPowerArgs};
    pub use crate::testing::*;
}
