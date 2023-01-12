use snapshot_trigger_service::client::args::{Error, TriggerServiceCliCommand};

use clap::Parser;
use futures::future::FutureExt;

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli_future = tokio::task::spawn_blocking(|| TriggerServiceCliCommand::parse().exec())
        .map(|res| res.expect("CLI command failed for an unknown reason"))
        .fuse();

    signals_handler::with_signal_handler(cli_future).await
}
