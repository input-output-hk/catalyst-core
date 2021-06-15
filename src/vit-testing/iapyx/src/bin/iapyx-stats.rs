use futures::future::FutureExt;
use iapyx::cli::args::stats::IapyxStatsCommand;
use structopt::StructOpt;

#[tokio::main]
pub async fn main() -> Result<(), iapyx::cli::args::stats::IapyxStatsCommandError> {
    std::env::set_var("RUST_BACKTRACE", "full");
    let cli_future = IapyxStatsCommand::from_args().exec().fuse();
    tokio::pin!(cli_future);
    signals_handler::with_signal_handler(cli_future).await
}
