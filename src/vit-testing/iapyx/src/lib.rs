/*#![forbid(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(
clippy::module_name_repetitions,
clippy::match_bool,
clippy::bool_assert_comparison,
clippy::derive_partial_eq_without_eq
)]*/

extern crate prettytable;
extern crate rand;

mod controller;
mod load;
pub mod utils;
mod wallet;

pub use crate::wallet::{Error as WalletError, Wallet};
pub use controller::{Controller, ControllerBuilder, ControllerBuilderError, ControllerError};
pub use load::{
    ArtificialUserLoad, ArtificialUserLoadError, MultiController, MultiControllerError, NodeLoad,
    NodeLoadConfig, NodeLoadError, ServicingStationLoad, ServicingStationLoadError,
    VoteStatusProvider, WalletRequestGen,
};
