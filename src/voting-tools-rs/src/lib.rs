//! Rust implementation of voting tools
//!
//! Original Haskell repository is <https://github.com/input-output-hk/voting-tools>

#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

#[macro_use]
extern crate tracing;

mod cli;
mod config;
mod db;
mod logic;
mod model;

pub use exports::*;
mod exports {
    pub use crate::cli::Args;
    pub use crate::config::DbConfig;
    pub use crate::logic::run;
}

#[cfg(test)]
mod testing;
