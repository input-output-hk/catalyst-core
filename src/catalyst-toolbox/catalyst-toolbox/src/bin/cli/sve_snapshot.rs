use std::{io::Write, path::PathBuf};

use chain_addr::Discrimination;
use clap::Parser;
use color_eyre::Report;
use jcli_lib::utils::{output_file::OutputFile, OutputFormat};
use jormungandr_lib::interfaces::{InitialUTxO, Value};
use serde::Serialize;
use snapshot_lib::{registration::VotingRegistration, RawSnapshot};

#[derive(Serialize)]
struct OutputInitial {
    fund: Vec<InitialUTxO>,
}

#[derive(Serialize)]
struct Output<'a> {
    initial: &'a [OutputInitial],
}

#[derive(Parser)]
pub struct SveSnapshotCmd {
    /// Input snapshot file path.
    #[clap(short, long)]
    file: PathBuf,

    /// Registrations voting power threshold for eligibility expressed in lovelace
    #[clap(short, long)]
    min_stake_threshold: Value,

    /// Discrimination to use for initial addresses
    #[clap(short, long)]
    discrimination: Discrimination,

    /// Whether voting power in the outputfile is expressed in lovelace or ada
    #[clap(short, long)]
    lovelace: bool,

    #[clap(flatten)]
    output: OutputFile,

    #[clap(flatten)]
    output_format: OutputFormat,
}

impl SveSnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
        // Make voting_purpose = 0 if voting_purpose is null in the original JSON file.
        let mut registratons: serde_json::Value =
            serde_json::from_reader(std::fs::File::open(&self.file)?)?;
        for r in registratons
            .as_array_mut()
            .expect("Expected input to be an array of voting registrations")
        {
            let voting_purpose = r
                .as_object_mut()
                .expect("Expected voting registration to be an object")
                .get_mut("voting_purpose")
                .expect("Missing voting_purpose key in voting registration");

            if voting_purpose.is_null() {
                *voting_purpose = serde_json::json!(0);
            }
        }

        // Filter CIP-36 registrations with more than 1 delegation.
        let registrations = serde_json::from_value::<Vec<VotingRegistration>>(registratons)?;

        let raw_snapshot = RawSnapshot::from(registrations);
        let (snapshot, total_registrations_rejected) =
            snapshot_lib::sve::Snapshot::new(raw_snapshot, self.min_stake_threshold);

        eprintln!("{} registrations rejected", total_registrations_rejected);

        let output = Output {
            initial: &[OutputInitial {
                fund: snapshot.to_block0_initials(self.discrimination, self.lovelace),
            }],
        };

        let mut out_writer = self.output.open()?;
        let content = self
            .output_format
            .format_json(serde_json::to_value(output)?)?;
        out_writer.write_all(content.as_bytes())?;

        Ok(())
    }
}
