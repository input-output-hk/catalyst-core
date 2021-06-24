pub use registration_service::{
    args::{Error, RegistrationServiceCommand},
    context::Context,
    utils::*,
};

use futures::future::FutureExt;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let cli_future = RegistrationServiceCommand::from_args().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
