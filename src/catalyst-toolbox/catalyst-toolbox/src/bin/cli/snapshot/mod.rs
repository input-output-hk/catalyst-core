use chain_addr::Discrimination;
use clap::Parser;
use color_eyre::Report;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::Value;
use snapshot_lib::{
    voting_group::{RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    Snapshot,
};
use snapshot_lib::{Dreps, Fraction};
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
    /// Path to the file containing all dreps information in json format.
    #[clap(long, value_parser = PathBuf::from_str)]
    dreps: Option<PathBuf>,
    /// Registrations voting power threshold for eligibility
    #[clap(short, long)]
    min_stake_threshold: Value,

    /// Voter group to assign direct voters to.
    /// If empty, defaults to "voter"
    #[clap(short, long)]
    direct_voters_group: Option<String>,

    /// Voter group to assign representatives to.
    /// If empty, defaults to "rep"
    #[clap(long)]
    representatives_group: Option<String>,

    /// Voting power cap for each account
    #[clap(short, long)]
    voting_power_cap: Fraction,

    #[clap(flatten)]
    output: OutputFile,

    #[clap(flatten)]
    output_format: OutputFormat,

    /// Discrimination to use for initial addresses
    #[clap(short, long, default_value = "production")]
    discrimination: Discrimination,
}

impl SnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        let raw_snapshot = serde_json::from_reader(File::open(&self.snapshot)?)?;
        let dreps = if let Some(dreps) = &self.dreps {
            serde_json::from_reader(File::open(dreps)?)?
        } else {
            Dreps::default()
        };
        let direct_voter = self
            .direct_voters_group
            .unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.into());
        let representative = self
            .representatives_group
            .unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.into());
        let assigner = RepsVotersAssigner::new(direct_voter, representative, dreps);
        let initials = Snapshot::from_raw_snapshot(
            raw_snapshot,
            self.min_stake_threshold,
            self.voting_power_cap,
            &assigner,
            self.discrimination,
        )?
        .to_full_snapshot_info();
        let mut out_writer = self.output.open()?;
        let content = self
            .output_format
            .format_json(serde_json::to_value(initials)?)?;
        out_writer.write_all(content.as_bytes())?;
        Ok(())
    }
}
