use super::config::Configuration;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::VitStartParameters;
use crate::mock::ledger_state::LedgerState;
use jormungandr_scenario_tests::prepare_command;
use jormungandr_scenario_tests::{Context, ProgressBarMode};
use jormungandr_testing_utils::testing::network::Seed;
use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;
use thiserror::Error;
use valgrind::VitVersion;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::data::Snapshot;
use vit_servicing_station_tests::common::data::ValidVotePlanGenerator;

pub struct MockState {
    pub available: bool,
    pub error_code: u16,
    version: VitVersion,
    ledger_state: LedgerState,
    vit_state: Snapshot,
}

pub fn context<P: AsRef<Path>>(testing_directory: P) -> Context {
    let jormungandr = prepare_command(PathBuf::from_str("jormungandr").unwrap());
    let jcli = prepare_command(PathBuf::from_str("jcli").unwrap());
    let seed = Seed::generate(rand::rngs::OsRng);
    let generate_documentation = true;
    let log_level = "info".to_string();

    Context::new(
        seed,
        jormungandr,
        jcli,
        Some(testing_directory.as_ref().to_path_buf()),
        generate_documentation,
        ProgressBarMode::None,
        log_level,
    )
}

impl MockState {
    pub fn new(params: VitStartParameters, config: Configuration) -> Result<Self, Error> {
        std::fs::remove_dir_all(&config.working_dir)?;

        let mut quick_setup = VitBackendSettingsBuilder::new();
        let context = context(&config.working_dir);
        quick_setup.upload_parameters(params);

        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
        let (_, controller, vit_parameters, version) = quick_setup.build(context).unwrap();

        let mut generator = ValidVotePlanGenerator::new(vit_parameters);
        let snapshot = generator.build(&mut template_generator);

        Ok(Self {
            available: true,
            error_code: 400,
            ledger_state: LedgerState::new(
                controller.settings().block0.clone(),
                controller.block0_file(),
            )?,
            vit_state: snapshot,
            version: VitVersion::new(version),
        })
    }

    pub fn version(&self) -> VitVersion {
        self.version.clone()
    }

    pub fn set_version(&mut self, version: String) {
        self.version = VitVersion::new(version);
    }

    pub fn vit(&self) -> &Snapshot {
        &self.vit_state
    }

    pub fn ledger(&self) -> &LedgerState {
        &self.ledger_state
    }

    pub fn ledger_mut(&mut self) -> &mut LedgerState {
        &mut self.ledger_state
    }

    pub fn set_fund_id(&mut self, id: i32) {
        let funds = self.vit_state.funds_mut();
        let mut fund = funds.last_mut().unwrap();

        fund.id = id;

        for challenge in fund.challenges.iter_mut() {
            challenge.fund_id = id;
        }

        for vote_plan in fund.chain_vote_plans.iter_mut() {
            vote_plan.fund_id = id;
        }

        for challenge in self.vit_state.challenges_mut() {
            challenge.fund_id = id;
        }

        for proposal in self.vit_state.proposals_mut() {
            proposal.proposal.fund_id = id;
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("ledger error")]
    LedgerError(#[from] super::ledger_state::Error),
    #[error("IO error")]
    IoError(#[from] std::io::Error),
}
