use catalyst_toolbox::snapshot::{RawSnapshot, Snapshot};
use chain_addr::Discrimination;
use color_eyre::Report;
use jcli_lib::utils::{output_file::OutputFile, output_format::OutputFormat};
use jormungandr_lib::interfaces::Value;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

/// Process raw registrations into blockchain initials
#[derive(StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SnapshotCmd {
    /// Path to the file containing all CIP-15 compatible registrations in json format.
    #[structopt(short, long, parse(from_os_str))]
    snapshot: PathBuf,
    /// Registrations voting power threshold for eligibility
    #[structopt(short, long)]
    threshold: Value,

    /// Discrimination to use for initial addresses
    #[structopt(short, long)]
    discrimination: Discrimination,

    #[structopt(flatten)]
    output: OutputFile,

    #[structopt(flatten)]
    output_format: OutputFormat,
}

impl SnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        let raw_snapshot: RawSnapshot = serde_json::from_reader(File::open(&self.snapshot)?)?;
        let initials = Snapshot::from_raw_snapshot(raw_snapshot, self.threshold)
            .to_block0_initials(self.discrimination);
        let mut out_writer = self.output.open()?;
        let content = self
            .output_format
            .format_json(serde_json::to_value(initials)?)?;
        out_writer.write_all(content.as_bytes())?;
        Ok(())
    }
}
