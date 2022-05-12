use catalyst_toolbox::stats::snapshot::read_initials;
use catalyst_toolbox::stats::voters::calculate_wallet_distribution_from_initials;
use catalyst_toolbox::stats::Error;
use jormungandr_lib::interfaces::Initial;
use std::path::PathBuf;
use structopt::StructOpt;
#[derive(StructOpt, Debug)]
pub struct SnapshotCommand {
    #[structopt(long = "support-lovelace")]
    pub support_lovelace: bool,
    #[structopt(name = "SNAPSHOT")]
    pub snapshot: PathBuf,
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

impl SnapshotCommand {
    pub fn exec(&self) -> Result<(), Error> {
        let initials: Vec<Initial> = read_initials(&jortestkit::file::read_file(&self.snapshot)?)?;

        match self.command {
            Command::Count => calculate_wallet_distribution_from_initials(
                initials,
                vec![],
                self.threshold,
                self.support_lovelace,
            )?
            .print_count_per_level(),
            Command::Ada => calculate_wallet_distribution_from_initials(
                initials,
                vec![],
                self.threshold,
                self.support_lovelace,
            )?
            .print_ada_per_level(),
        };

        Ok(())
    }
}
