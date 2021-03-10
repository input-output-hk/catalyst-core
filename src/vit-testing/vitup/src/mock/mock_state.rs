use crate::config::VitStartParameters;
use crate::mock::ledger_state::LedgerState;
use crate::setup::start::quick::QuickVitBackendSettingsBuilder;
use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::Context;
use jormungandr_testing_utils::testing::network_builder::Seed;
use jortestkit::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::Snapshot;
use vit_servicing_station_tests::common::data::ValidVotePlanGenerator;

pub struct MockState {
    pub available: bool,
    ledger_state: LedgerState,
    vit_state: Snapshot,
}

pub fn context(testing_directory: &PathBuf) -> Context {
    let jormungandr = prepare_command(PathBuf::from_str("jormungandr").unwrap());
    let jcli = prepare_command(PathBuf::from_str("jcli").unwrap());
    let seed = Seed::generate(rand::rngs::OsRng);
    let generate_documentation = true;
    let log_level = "info".to_string();

    Context::new(
        seed,
        jormungandr,
        jcli,
        Some(testing_directory.clone()),
        generate_documentation,
        ProgressBarMode::None,
        log_level,
    )
}

impl MockState {
    pub fn new<P: AsRef<Path>>(
        params: VitStartParameters,
        testing_directory: P,
    ) -> Result<Self, Error> {
        let mut quick_setup = QuickVitBackendSettingsBuilder::new();
        let context = context(&testing_directory.as_ref().to_path_buf());
        quick_setup.upload_parameters(params);
        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
        let (_, controller, vit_parameters) = quick_setup.build(context).unwrap();

        let mut generator = ValidVotePlanGenerator::new(vit_parameters);
        let snapshot = generator.build(&mut template_generator);

        Ok(Self {
            available: true,
            ledger_state: LedgerState::new(
                controller.settings().network_settings.block0.clone(),
                controller.block0_file(),
            )?,
            vit_state: snapshot,
        })
    }

    pub fn vit(&self) -> &Snapshot {
        &self.vit_state
    }

    pub fn vit_mut(&mut self) -> &mut Snapshot {
        &mut self.vit_state
    }

    pub fn ledger(&self) -> &LedgerState {
        &self.ledger_state
    }

    pub fn ledger_mut(&mut self) -> &mut LedgerState {
        &mut self.ledger_state
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("ledger error")]
    LedgerError(#[from] super::ledger_state::Error),
}
