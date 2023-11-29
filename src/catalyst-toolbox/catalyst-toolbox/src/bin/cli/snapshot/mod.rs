use chain_addr::Discrimination;
use clap::Parser;
use color_eyre::Report;
use itertools::Itertools;
use jcli_lib::utils::output_file::OutputFile;
use jormungandr_lib::interfaces::Value;

use serde::Serialize;
use snapshot_lib::{
    voting_group::{RepsVotersAssigner, DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP},
    Snapshot, SnapshotInfo,
};
use snapshot_lib::{Dreps, Fraction};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::info;

/// Process raw registrations into blockchain initials
#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub struct SnapshotCmd {
    /// Base file to save.  Will also create a <base_file>.summary<.extension> file.
    #[clap(flatten)]
    output: OutputFile,

    /// Path to the file containing all CIP-15 compatible registrations in json format.
    #[clap(short, long, value_parser = PathBuf::from_str)]
    snapshot: PathBuf,

    /// Discrimination to use for initial addresses
    #[clap(short, long, default_value = "production")]
    discrimination: Discrimination,

    // Processing Options
    /// Registrations voting power threshold for eligibility
    #[clap(short, long)]
    min_stake_threshold: Value,

    /// Voting power cap for each account
    #[clap(short, long, default_value = "100.0")]
    voting_power_cap: Fraction,

    /// Make a loadtest suitable snapshot.
    #[clap(short, long, default_value = "false")]
    loadtest: bool,

    //processing: SnapshotCmdProcessingOptions,
    /// What was the registration deadline date-time the snapshot is aiming for?
    #[clap(long, default_value = "Unknown")]
    deadline_datetime: String,

    /// What Slot does this snapshot align with.
    #[clap(long)]
    slot_no: Option<u64>,

    /// What is the date-time of the snapshot alignment slot. (RFC3399 Formatted)
    #[clap(long, default_value = "Unknown")]
    slot_datetime: String,

    /// What was the tip of the chain when this snapshot was run.
    #[clap(long)]
    tip_slot_no: Option<u64>,

    /// What was the slot date-time of tip when this snapshot was run. (RFC3399 Formatted)
    #[clap(long, default_value = "Unknown")]
    tip_slot_datetime: String,

    /// What was the registration deadline slot_no the snapshot is aiming for?
    #[clap(long)]
    deadline_slot_no: Option<u64>,

    /// What was the actual slot date-time of the registration deadline is. (RFC3399 Formatted)
    #[clap(long, default_value = "Unknown")]
    deadline_slot_datetime: String,

    /// Is this a non-final snapshot.
    #[clap(short, default_value = "false")]
    final_snapshot: bool,

    /// Path to the file containing all dreps information in json format.
    /// Currently Unsupported
    #[clap(long, value_parser = PathBuf::from_str)]
    dreps: Option<PathBuf>,
    /// Voter group to assign direct voters to.
    /// If empty, defaults to "voter"
    /// Currently Unsupported
    #[clap(long)]
    direct_voters_group: Option<String>,
    /// Voter group to assign representatives to.
    /// If empty, defaults to "rep"
    /// Currently Unsupported
    #[clap(long)]
    representatives_group: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !(*b)
}

#[derive(Serialize)]
pub struct SnapshotConfig {
    // Type of snapshot info
    #[serde(skip_serializing_if = "is_false")]
    load_test: bool,

    // Parameters we processed with
    discrimination: Discrimination,
    min_stake_threshold: Value,
    voting_power_cap_pct: String,
    voting_power_cap: u64,

    // What the snapshot represents
    deadline_datetime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    slot_no: Option<u64>,
    slot_datetime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tip_slot_no: Option<u64>,
    tip_slot_datetime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    deadline_slot_no: Option<u64>,
    deadline_slot_datetime: String,
    interim_snapshot: bool,

    // Summary data from the snapshot processing.
    total_registered_voters: u64,
    total_registered_voting_power: u128,
    total_eligible_voters: u64,
    total_eligible_voting_power: u128,
}

#[derive(Serialize)]
pub struct SnapshotReport {
    config: SnapshotConfig,
    voters: Vec<SnapshotInfo>,
}

#[derive(Serialize)]
pub struct SnapshotSummaryVoter {
    address: String,
    value: u64,
}

#[derive(Serialize)]
pub struct SnapshotSummaryFund {
    fund: Vec<SnapshotSummaryVoter>,
}

#[derive(Serialize)]
pub struct SnapshotSummaryReport {
    initial: Vec<SnapshotSummaryFund>,
}

impl SnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        if self.voting_power_cap > Fraction::from(100) {
            return Err(color_eyre::eyre::eyre!(
                "Voting power cap (%) must be less than 100.0 "
            ));
        } else if self.voting_power_cap < Fraction::from(0) {
            return Err(color_eyre::eyre::eyre!(
                "Voting power cap (%) must be greater than 0.0"
            ));
        }

        info!("Reading Raw Snapshot");

        // Voting Power cap is a percentage so rebase it to 1, not 100.
        let voting_power_cap = self.voting_power_cap / 100.0;

        // Serde_json::from_reader is glacially slow.  Read as string first.
        // See: https://github.com/serde-rs/json/issues/160
        // serde_json::from_reader took 28 seconds to read the file.
        // Reading to a  string and then converting took 90ms.
        let raw_snapshot_data = std::fs::read_to_string(&self.snapshot)?;

        let raw_snapshot = serde_json::from_str(&raw_snapshot_data)?;
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

        info!("Processing Snapshot");

        let processed_snapshot = Snapshot::from_raw_snapshot(
            raw_snapshot,
            self.min_stake_threshold,
            voting_power_cap,
            &assigner,
            self.discrimination,
            self.loadtest,
        )?;

        info!("Generating Report");

        let report = SnapshotReport {
            config: SnapshotConfig {
                load_test: self.loadtest,
                discrimination: self.discrimination,
                min_stake_threshold: processed_snapshot.stake_threshold,
                voting_power_cap_pct: format!("{:.8}", self.voting_power_cap),
                voting_power_cap: processed_snapshot.voting_power_cap,

                deadline_datetime: self.deadline_datetime,
                slot_no: self.slot_no,
                slot_datetime: self.slot_datetime,
                tip_slot_no: self.tip_slot_no,
                tip_slot_datetime: self.tip_slot_datetime,
                deadline_slot_no: self.deadline_slot_no,
                deadline_slot_datetime: self.deadline_slot_datetime,
                interim_snapshot: !self.final_snapshot,

                total_registered_voters: processed_snapshot.total_registered_voters,
                total_registered_voting_power: processed_snapshot.total_registered_voting_power,
                total_eligible_voters: processed_snapshot.total_eligible_voters,
                total_eligible_voting_power: processed_snapshot.total_eligible_voting_power,
            },
            voters: processed_snapshot.to_full_snapshot_info(),
        };

        // Write the primary processed snapshot report.
        let mut out_writer = self.output.open()?;
        let content = serde_json::to_string_pretty(&report)?;
        out_writer.write_all(content.as_bytes())?;

        info!("Generating Summary");

        // Write the Summary we use for vit-ss compatibility.
        // Sorted by voter key so that the report is reproducible deterministically.

        let report_data = report
            .voters
            .iter()
            .sorted_by_key(|v| v.hir.address.clone())
            .filter(|v| !v.hir.underthreshold) // Summary does not have voters who don't have enough voting power.
            .map(|v| SnapshotSummaryVoter {
                address: v.hir.address.to_string(),
                value: v.hir.voting_power.as_u64() / 1000000, // Lovelace to Whole ADA conversion.
            })
            .collect();

        let summary_report = SnapshotSummaryReport {
            initial: vec![SnapshotSummaryFund { fund: report_data }],
        };
        let summary_output = self.output.extension_prefix("summary");
        let mut out_writer = summary_output.open()?;
        let content = serde_json::to_string_pretty(&summary_report)?;
        out_writer.write_all(content.as_bytes())?;

        info!("Snapshot Processing Completed OK");

        Ok(())
    }
}
