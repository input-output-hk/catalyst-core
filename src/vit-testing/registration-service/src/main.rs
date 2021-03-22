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
use structopt::StructOpt;
pub use utils::*;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    RegistrationServiceCommand::from_args().exec()
}
