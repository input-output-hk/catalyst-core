use catalyst_toolbox::stats::voters::calculate_wallet_distribution;
use color_eyre::Report;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct InitialVotersCommand {
    #[structopt(long = "support-lovelace")]
    pub support_lovelace: bool,
    #[structopt(long = "block0")]
    pub block0: String,
    #[structopt(long = "threshold")]
    pub threshold: u64,
    #[structopt(subcommand)]
    pub command: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Count,
    Ada,
}

impl InitialVotersCommand {
    pub fn exec(&self) -> Result<(), Report> {
        match self.command {
            Command::Count => {
                calculate_wallet_distribution(&self.block0, self.threshold, self.support_lovelace)?
                    .print_count_per_level()
            }
            Command::Ada => {
                calculate_wallet_distribution(&self.block0, self.threshold, self.support_lovelace)?
                    .print_ada_per_level()
            }
        };

        Ok(())
    }
}
