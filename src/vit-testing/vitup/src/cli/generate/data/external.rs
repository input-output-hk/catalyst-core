use crate::builders::utils::{logger, DeploymentTree, SessionSettingsExtension};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::read_config;
use crate::mode::standard::generate_database;
use crate::Result;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use std::path::PathBuf;
use structopt::StructOpt;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct ExternalDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// configuration
    #[structopt(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[structopt(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[structopt(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[structopt(long = "funds", default_value = "../resources/external/funds.json")]
    pub funds: PathBuf,

    /// reviews import json
    #[structopt(long = "reviews", default_value = "../resources/external/reviews.json")]
    pub reviews: PathBuf,

    #[structopt(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[structopt(long = "skip-qr-generation")]
    pub skip_qr_generation: bool,

    #[structopt(long = "log-level", default_value = "LogLevel::Info")]
    pub log_level: LogLevel,
}

impl ExternalDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        logger::init(self.log_level)?;

        let session_settings = SessionSettings::from_dir(&self.output_directory);

        let mut quick_setup = VitBackendSettingsBuilder::default();
        let mut config = read_config(&self.config)?;

        if let Some(snapshot) = self.snapshot {
            config.extend_from_initials_file(snapshot, chain_addr::Discrimination::Production)?;
        }

        if self.skip_qr_generation {
            quick_setup = quick_setup.skip_qr_generation();
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory);

        let (controller, vit_parameters) = quick_setup
            .config(&config)
            .session_settings(session_settings)
            .build()?;

        let template_generator = ExternalValidVotingTemplateGenerator::new(
            self.proposals,
            self.challenges,
            self.funds,
            self.reviews,
        )
        .unwrap();

        generate_database(&deployment_tree, vit_parameters, template_generator)?;

        config.print_report(Some(controller));
        Ok(())
    }
}
