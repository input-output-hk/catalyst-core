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
    let cli_future = RegistrationServiceCommand::from_args().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
