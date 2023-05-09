use std::{io::Write, path::PathBuf};

use chain_addr::Discrimination;
use clap::Parser;
use color_eyre::Report;
use fraction::Fraction;
use jcli_lib::utils::{output_file::OutputFile, OutputFormat};
use jormungandr_lib::interfaces::Value;
use snapshot_lib::{
    registration::VotingRegistration, voting_group::VotingGroupAssigner, RawSnapshot, Snapshot,
};

struct DummyAssigner;

impl VotingGroupAssigner for DummyAssigner {
    fn assign(
        &self,
        _: &jormungandr_lib::crypto::account::Identifier,
    ) -> snapshot_lib::VotingGroup {
        "".to_string()
    }
}

#[derive(Parser)]
pub struct SveSnapshotCmd {
    /// Input snapshot file path.
    #[clap(short, long)]
    file: PathBuf,

    /// Registrations voting power threshold for eligibility
    #[clap(short, long)]
    min_stake_threshold: Value,

    /// Voting power cap for each account
    #[clap(short, long)]
    voting_power_cap: Fraction,

    /// Discrimination to use for initial addresses
    #[clap(short, long)]
    discrimination: Discrimination,

    #[clap(flatten)]
    output: OutputFile,

    #[clap(flatten)]
    output_format: OutputFormat,
}

impl SveSnapshotCmd {
    pub fn exec(self) -> Result<(), Report> {
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

        let registrations: Vec<VotingRegistration> = serde_json::from_value(registratons)?;
        let processed_registrations = registrations
            .into_iter()
            .filter(|r| {
                if let snapshot_lib::registration::Delegations::New(ds) = &r.delegations {
                    if ds.len() != 1 {
                        return false;
                    }
                }

                true
            })
            .collect::<Vec<_>>();

        let raw_snapshot = RawSnapshot::from(processed_registrations);
        let dummy_assigner = DummyAssigner;
        let snapshot = Snapshot::from_raw_snapshot(
            raw_snapshot,
            self.min_stake_threshold,
            self.voting_power_cap,
            &dummy_assigner,
        )?;

        let initials = snapshot.to_block0_initials(chain_addr::Discrimination::Test);

        let mut out_writer = self.output.open()?;
        let content = self
            .output_format
            .format_json(serde_json::to_value(initials)?)?;
        out_writer.write_all(content.as_bytes())?;

        Ok(())
    }
}
