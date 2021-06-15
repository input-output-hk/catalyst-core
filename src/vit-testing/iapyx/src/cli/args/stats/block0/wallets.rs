use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::block0::wallets::calculate_wallet_distribution;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct WalletsCommand {
    #[structopt(long = "block0")]
    pub block0: String,
    #[structopt(long = "threshold")]
    pub threshold: u64,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Distribution,
}

impl WalletsCommand {
    pub fn exec(&self) -> Result<(), IapyxStatsCommandError> {
        match self.command {
            Command::Distribution => calculate_wallet_distribution(&self.block0, self.threshold),
        }
    }
}
