use futures::future::FutureExt;
use mainnet_tools::voter_registration::Command;
use clap::Parser;

#[tokio::main]
pub async fn main() -> Result<(), mainnet_tools::voter_registration::Error> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli_future = tokio::task::spawn_blocking(|| Command::parse().exec())
        .map(|res| res.expect("CLI command failed for an unknown reason"))
        .fuse();

    signals_handler::with_signal_handler(cli_future).await
}
