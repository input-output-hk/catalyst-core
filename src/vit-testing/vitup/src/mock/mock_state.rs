use super::config::Configuration;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::VitStartParameters;
use crate::mock::ledger_state::LedgerState;
use hersir::controller::Context;
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
    let jormungandr = PathBuf::from_str("jormungandr").unwrap();
    let generate_documentation = true;

    Context {
        jormungandr,
        testing_directory: testing_directory.as_ref().to_path_buf().into(),
        generate_documentation,
        session_mode: todo!("session mode?"),
    }
}

impl MockState {
    pub fn new(params: VitStartParameters, config: Configuration) -> Result<Self, Error> {
        if config.working_dir.exists() {
            std::fs::remove_dir_all(&config.working_dir)?;
        }
        let mut quick_setup = VitBackendSettingsBuilder::new();
        let context = context(&config.working_dir);
        quick_setup.upload_parameters(params);

        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
        let (_, controller, vit_parameters, version) = quick_setup.build(context).unwrap();

        let mut generator = ValidVotePlanGenerator::new(vit_parameters);
        let mut snapshot = generator.build(&mut template_generator);

        let reviews = snapshot.advisor_reviews();

        //perform db view operations
        for proposal in snapshot.proposals_mut().iter_mut() {
            proposal.proposal.reviews_count = reviews
                .iter()
                .filter(|review| review.proposal_id.to_string() == proposal.proposal.proposal_id)
                .count() as i32;
        }

        Ok(Self {
            available: true,
            error_code: 400,
            ledger_state: LedgerState::new(
                controller.settings().block0.clone(),
                controller.block0_file(),
            )?,
            vit_state: snapshot,
            version: VitVersion {
                service_version: version,
            },
        })
    }

    pub fn version(&self) -> VitVersion {
        VitVersion {
            service_version: self.version.service_version.clone(),
        }
    }

    pub fn set_version(&mut self, version: String) {
        self.version = VitVersion {
            service_version: version,
        }
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
