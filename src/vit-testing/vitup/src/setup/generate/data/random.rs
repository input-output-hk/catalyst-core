use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;

use super::{encode, read_config, read_genesis_yaml, write_genesis_yaml};
use crate::setup::generate::read_initials;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct RandomDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[structopt(long = "config")]
    pub config: PathBuf,

    #[structopt(long = "snapshot")]
    pub snapshot: Option<PathBuf>,
}

impl RandomDataCommandArgs {
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
        let mut config = read_config(&self.config)?;

        if let Some(snapshot) = self.snapshot {
            config
                .params
                .initials
                .extend_from_external(read_initials(snapshot)?);
        }

        quick_setup.upload_parameters(config.params.clone());
        quick_setup.fees(config.linear_fees);
        quick_setup.set_external_committees(config.committees);

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let title = quick_setup.title();

        let (vit_controller, mut controller, vit_parameters, version) =
            quick_setup.build(context)?;
        let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

        // generate vit station data
        let vit_station = vit_controller.spawn_vit_station(
            &mut controller,
            vit_parameters,
            &mut template_generator,
            version,
        )?;
        vit_station.shutdown();

        let mut root_directory = self.output_directory;
        root_directory.push(title);

        let mut genesis = root_directory.clone();
        genesis.push("genesis.yaml");

        let mut block0 = root_directory;
        block0.push("block0.bin");

        let mut block0_configuration = read_genesis_yaml(&genesis)?;

        if !config.consensus_leader_ids.is_empty() {
            block0_configuration
                .blockchain_configuration
                .consensus_leader_ids = config.consensus_leader_ids;
        }
        write_genesis_yaml(block0_configuration, &genesis)?;
        encode(&genesis, &block0)?;

        println!(
            "voteplan ids: {:?}",
            controller
                .vote_plans()
                .iter()
                .map(|x| x.id())
                .collect::<Vec<String>>()
        );

        quick_setup.print_report();
        Ok(())
    }
}
