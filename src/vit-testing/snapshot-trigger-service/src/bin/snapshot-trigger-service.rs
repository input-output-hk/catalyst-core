use snapshot_trigger_service::{Error, TriggerServiceCommand};

use futures::future::FutureExt;
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let cli_future = TriggerServiceCommand::from_args().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
