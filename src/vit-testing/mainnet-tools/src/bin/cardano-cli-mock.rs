use futures::future::FutureExt;
use mainnet_tools::cardano_cli::{CardanoCliCommand, CardanoCliCommandError};
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), CardanoCliCommandError> {
    std::env::set_var("RUST_BACKTRACE", "full");

    let cli_future = tokio::task::spawn_blocking(|| CardanoCliCommand::from_args().exec())
        .map(|res| res.expect("CLI command failed for an unknown reason"))
        .fuse();

    signals_handler::with_signal_handler(cli_future).await
}
