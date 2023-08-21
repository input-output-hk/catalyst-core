//! Rust implementation of voting tools
//!
//! Original Haskell repository is <https://github.com/input-output-hk/voting-tools>
//!
//! The queries themselves (as well as the details of the CLI) are different, but they should
//! produce similar outputs. Malformed registrations are silently ignored.

#![warn(clippy::pedantic)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
// #![deny(clippy::integer_arithmetic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq,
    clippy::missing_panics_doc,
    clippy::match_on_vec_items,
    clippy::unnecessary_wraps,
    clippy::cast_sign_loss,
    clippy::iter_nth_zero,
    clippy::type_complexity,
    clippy::match_same_arms,
    clippy::useless_conversion,
    clippy::wildcard_imports
)]
#![cfg_attr(test, allow(let_underscore_drop))] // useful in tests, often a bug otherwise

#[macro_use]
extern crate tracing;

mod cli;
mod data;
// mod data_provider;
mod db;
mod error;
mod logic;
mod testing;
pub mod verification;

// this export style forces us to be explicit about what is in the public API
pub use exports::*;
mod exports {
    pub use crate::cli::{show_error_warning, Args, DryRunCommand};
    pub use crate::data::{Sig, Signature, SlotNo, SnapshotEntry, VotingKey, VotingPurpose};
    pub use crate::db::DbConfig;
    pub use crate::error::*;
    pub use crate::logic::{voting_power, VotingPowerArgs};
    pub use crate::testing::*;
    pub use crate::verification::*;
}
