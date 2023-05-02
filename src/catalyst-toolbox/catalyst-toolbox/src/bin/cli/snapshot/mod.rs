use clap::Parser;
use color_eyre::Report;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::Value;
use snapshot_lib::Fraction;
use snapshot_lib::{
    voting_group::{RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    RawSnapshot, Snapshot,
};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// Process raw registrations into blockchain initials
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct SnapshotCmd {
    /// Path to the file containing all CIP-15 compatible registrations in json format.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    snapshot: PathBuf,

    /// Registrations voting power threshold for eligibility
    #[clap(short, long)]
    min_stake_threshold: Value,

    /// Voter group to assign direct voters to.
    #[clap(short, long, default_value_t = DEFAULT_DIRECT_VOTER_GROUP.to_string())]
    direct_voters_group: String,

    /// Voter group to assign representatives to.
    #[clap(long, default_value_t = DEFAULT_REPRESENTATIVE_GROUP.to_string())]
    representatives_group: String,

    /// Path to a file containing the list of representatives.
    #[clap(long)]
    reps_file: Option<PathBuf>,

    /// Voting power cap for each account
    #[clap(short, long)]
    voting_power_cap: Fraction,

    #[clap(flatten)]
    output: OutputFile,

    #[clap(flatten)]
    output_format: OutputFormat,
}

impl SnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        let raw_snapshot: RawSnapshot = serde_json::from_reader(File::open(&self.snapshot)?)?;

        let direct_voter = self.direct_voters_group;
        let representative = self.representatives_group;

        let assigner = if let Some(file_path) = self.reps_file.as_ref() {
            RepsVotersAssigner::new_from_reps_file(direct_voter, representative, file_path)?
        } else {
            RepsVotersAssigner::new(direct_voter, representative)
        };

        let initials = Snapshot::from_raw_snapshot(
            raw_snapshot,
            self.min_stake_threshold,
            self.voting_power_cap,
            &assigner,
        )?
        .to_full_snapshot_info();

        let content = self
            .output_format
            .format_json(serde_json::to_value(initials)?)?;

        let mut out_writer = self.output.open()?;
        out_writer.write_all(content.as_bytes())?;
        Ok(())
    }
}
