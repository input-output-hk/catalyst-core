use color_eyre::Report;
use fraction::Fraction;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::Value;
use snapshot_lib::{voting_group::RepsVotersAssigner, RawSnapshot, Snapshot};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

const DEFAULT_DIRECT_VOTER_GROUP: &str = "direct";
const DEFAULT_REPRESENTATIVE_GROUP: &str = "rep";

/// Process raw registrations into blockchain initials
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SnapshotCmd {
    /// Path to the file containing all CIP-15 compatible registrations in json format.
    #[structopt(short, long, parse(from_os_str))]
    snapshot: PathBuf,
    /// Registrations voting power threshold for eligibility
    #[structopt(short, long)]
    min_stake_threshold: Value,

    /// Voter group to assign direct voters to.
    /// If empty, defaults to "voter"
    #[structopt(short, long)]
    direct_voters_group: Option<String>,

    /// Voter group to assign representatives to.
    /// If empty, defaults to "rep"
    #[structopt(long)]
    representatives_group: Option<String>,

    /// Url of the representative db api server
    #[structopt(long)]
    reps_db_api_url: reqwest::Url,

    /// Voting power cap for each account
    #[structopt(short, long)]
    voting_power_cap: Fraction,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,
}

impl SnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        let raw_snapshot: RawSnapshot = serde_json::from_reader(File::open(&self.snapshot)?)?;
        let direct_voter = self
            .direct_voters_group
            .unwrap_or_else(|| DEFAULT_DIRECT_VOTER_GROUP.into());
        let representative = self
            .representatives_group
            .unwrap_or_else(|| DEFAULT_REPRESENTATIVE_GROUP.into());
        let assigner = RepsVotersAssigner::new(direct_voter, representative, self.reps_db_api_url)?;
        let initials = Snapshot::from_raw_snapshot(
            raw_snapshot,
            self.min_stake_threshold,
            self.voting_power_cap,
            &assigner,
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
