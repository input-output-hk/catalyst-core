use catalyst_toolbox::stats::distribution::Stats;
use catalyst_toolbox::stats::voters::calculate_active_wallet_distribution;
use color_eyre::Report;
use std::ops::Range;
use std::path::PathBuf;
use clap::Parser;
use thiserror::Error;

#[derive(Parser, Debug)]
pub struct ActiveVotersCommand {
    #[clap(long = "support-lovelace")]
    pub support_lovelace: bool,
    #[clap(long = "block0")]
    pub block0: String,
    #[clap(long = "threshold")]
    pub threshold: u64,
    #[clap(long = "votes-count-file")]
    pub votes_count_path: PathBuf,
    #[clap(long = "votes-count-levels")]
    pub votes_count_levels: Option<PathBuf>,
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Parser, Debug)]
pub enum Command {
    Count,
    Ada,
    Votes,
}

impl ActiveVotersCommand {
    pub fn exec(&self) -> Result<(), Report> {
        match self.command {
            Command::Count => calculate_active_wallet_distribution(
                Stats::new(self.threshold)?,
                &self.block0,
                &self.votes_count_path,
                self.support_lovelace,
                |stats, value, _| stats.add(value),
            )?
            .print_count_per_level(),
            Command::Ada => calculate_active_wallet_distribution(
                Stats::new(self.threshold)?,
                &self.block0,
                &self.votes_count_path,
                self.support_lovelace,
                |stats, value, _| stats.add(value),
            )?
            .print_ada_per_level(),
            Command::Votes => calculate_active_wallet_distribution(
                Stats::new_with_levels(get_casted_votes_levels(&self.votes_count_levels)?),
                &self.block0,
                &self.votes_count_path,
                self.support_lovelace,
                |stats, _, weight| stats.add_with_weight(1, weight as u32),
            )?
            .print_count_per_level(),
        };

        Ok(())
    }
}

fn get_casted_votes_levels(path: &Option<PathBuf>) -> Result<Vec<Range<u64>>, Error> {
    if let Some(path) = path {
        serde_json::from_reader(jcli_lib::utils::io::open_file_read(&Some(path))?)
            .map_err(Into::into)
    } else {
        Ok(default_casted_votes_levels())
    }
}

fn default_casted_votes_levels() -> Vec<Range<u64>> {
    vec![
        (1..5),
        (5..10),
        (10..20),
        (20..50),
        (50..100),
        (100..200),
        (200..400),
        (400..800),
        (800..5_000),
    ]
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
