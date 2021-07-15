mod voters;
mod wallets;

use crate::cli::args::stats::IapyxStatsCommandError;
use structopt::StructOpt;
use voters::VotersCommand;
use wallets::WalletsCommand;

#[derive(StructOpt, Debug)]
pub enum Block0StatsCommand {
    Wallets(WalletsCommand),
    Voters(VotersCommand),
}

impl Block0StatsCommand {
    pub fn exec(&self) -> Result<(), IapyxStatsCommandError> {
        match self {
            Self::Wallets(wallets) => wallets.exec(),
            Self::Voters(voters) => voters.exec(),
        }
    }
}
