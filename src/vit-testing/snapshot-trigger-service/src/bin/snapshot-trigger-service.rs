use snapshot_trigger_service::{Error, TriggerServiceCommand};

use clap::Parser;
use futures::future::FutureExt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let cli_future = TriggerServiceCommand::parse().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
