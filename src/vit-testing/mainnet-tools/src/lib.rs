//! Toolbox for testing and mocking Cardano haskell part of catalyst bridge.

#![forbid(missing_docs)]
#![warn(clippy::pedantic)]

#[macro_use]
extern crate prettytable;
extern crate core;

#[allow(dead_code)]
/// Cardano CLI wrapper and mock
pub mod cardano_cli;
/// Simple scheduler for transporting snapshot from snapshot trigger service to servicing station service
/// Should be discarded when production component will be ready
pub mod snapshot_wormhole;
/// Mock for voter registration. It can produce mocked catalyst registration.
/// TODO: replace with mainnet wallet mock as it can produce real not mock catalyst registration
pub mod voter_registration;

