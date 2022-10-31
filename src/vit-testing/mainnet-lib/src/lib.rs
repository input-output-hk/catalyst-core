//! Rust implementation of Cardano various tools
//!
//! [`DbSyncInstance`] - mock of db sync based on json file rather than postgres db
//! Network - mock of cardano network in memory
//! Wallet - api implementation of cardano wallet which is capable of sending and signing registration
//! transactions

#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq
)]

mod db_sync;
mod network;
mod wallet;

pub use db_sync::DbSyncInstance;
pub use network::{
    MainnetNetwork, MainnetNetworkBuilder, MainnetWalletState, MainnetWalletStateBuilder,
};
pub use wallet::MainnetWallet;
