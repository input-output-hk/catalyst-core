use crate::cli::args::stats::IapyxStatsCommandError;
use crate::stats::block0::voters::count_active_voters;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct VotersCommand {
    #[structopt(long = "endpoint")]
    pub endpoint: String,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Count,
}

impl VotersCommand {
    pub fn exec(&self) -> Result<(), IapyxStatsCommandError> {
        match self.command {
            Command::Count => count_active_voters(&self.endpoint),
        }
    }
}
