//! Library part of iapyx which serves as a core lib for this testing CLI wallet.
//! Idea behind split was to enable api interface for integration tests without spawning CLI process
//! but directly operates on API level
#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
    clippy::module_name_repetitions,
    clippy::match_bool,
    clippy::bool_assert_comparison,
    clippy::derive_partial_eq_without_eq
)]
extern crate rand;

mod controller;
mod load;
mod utils;
mod wallet;

// this export style forces us to be explicit about what is in the public API
pub use exports::*;
mod exports {
    pub use crate::controller::{
        Controller, ControllerBuilder, ControllerBuilderError, ControllerError,
    };
    pub use crate::load::{
        config::NodeLoadConfig, ArtificialUserLoad, ArtificialUserLoadError, MultiController,
        MultiControllerError, NodeLoad, NodeLoadError, ServicingStationLoad,
        ServicingStationLoadError, VoteStatusProvider, WalletRequestGen,
    };

    pub use crate::utils::{expiry, qr};
    pub use crate::wallet::{Error as WalletError, Wallet};
}
