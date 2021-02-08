use super::{encode, read_config, read_genesis_yaml, write_genesis_yaml};
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct ExternalDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[structopt(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[structopt(long = "proposals")]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(long = "challenges")]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(long = "funds")]
    pub funds: PathBuf,
}

impl ExternalDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let context = Context::new(
            Seed::generate(rand::rngs::OsRng),
            PathBuf::new(),
            PathBuf::new(),
            Some(self.output_directory.clone()),
            true,
            ScenarioProgressBarMode::None,
            "info".to_string(),
        );

        let mut quick_setup = QuickVitBackendSettingsBuilder::new();
        let config = read_config(&self.config)?;

        quick_setup.upload_parameters(config.params.clone());

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let title = quick_setup.title();
        let (vit_controller, mut controller, vit_parameters) = quick_setup.build(context)?;

        let mut template_generator =
            ExternalValidVotingTemplateGenerator::new(self.proposals, self.challenges, self.funds)
                .unwrap();

        // generate vit station data
        let vit_station = vit_controller.spawn_vit_station(
            &mut controller,
            vit_parameters,
            &mut template_generator,
        )?;
        vit_station.shutdown();

        let mut root_directory = self.output_directory;
        root_directory.push(title);

        let mut genesis = root_directory.clone();
        genesis.push("genesis.yaml");

        let mut block0 = root_directory;
        block0.push("block0.bin");

        let mut block0_configuration = read_genesis_yaml(&genesis)?;

        block0_configuration.blockchain_configuration.linear_fees = config.linear_fees;
        if !config.consensus_leader_ids.is_empty() {
            block0_configuration
                .blockchain_configuration
                .consensus_leader_ids = config.consensus_leader_ids;
        }
        if !config.committees.is_empty() {
            block0_configuration
                .blockchain_configuration
                .committees
                .extend(config.committees.clone());
        }
        if !config.additions.is_empty() {
            block0_configuration.initial.extend(config.additions);
        }

        write_genesis_yaml(block0_configuration, &genesis)?;
        println!("genesis.yaml: {:?}", std::fs::canonicalize(&genesis)?);
        encode(&genesis, &block0)?;
        println!("block0: {:?}", std::fs::canonicalize(&block0)?);

        Ok(())
    }
}
