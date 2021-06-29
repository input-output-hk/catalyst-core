pub use registration_verify_service::{
    args::{Error, RegistrationVerifyServiceCommand},
    context::Context,
};

use futures::future::FutureExt;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let cli_future = RegistrationVerifyServiceCommand::from_args().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
