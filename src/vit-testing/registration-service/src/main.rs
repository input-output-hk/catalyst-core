mod args;
mod config;
mod context;
mod file_lister;
mod job;
mod request;
mod rest;
mod service;
mod utils;

pub use args::{Error, RegistrationServiceCommand};
pub use context::Context;
pub use utils::*;

use futures::future::FutureExt;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli_future = tokio::task::spawn_blocking(|| RegistrationServiceCommand::from_args().exec())
        .map(|res| res.expect("CLI command failed for an unknown reason"))
        .fuse();

    signals_handler::with_signal_handler(cli_future).await
}
