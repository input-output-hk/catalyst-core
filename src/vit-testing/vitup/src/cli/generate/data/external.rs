use crate::builders::utils::{logger, DeploymentTree, SessionSettingsExtension};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::read_config;
use crate::mode::standard::generate_database;
use crate::Result;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use std::path::PathBuf;
use clap::Parser;
use vit_servicing_station_tests::common::data::ExternalValidVotingTemplateGenerator;

#[derive(Parser, Debug)]
pub struct ExternalDataCommandArgs {
    /// Careful! directory would be removed before export
    #[clap(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// configuration
    #[clap(long = "config")]
    pub config: PathBuf,

    /// proposals import json
    #[clap(
        long = "proposals",
        default_value = "../resources/external/proposals.json"
    )]
    pub proposals: PathBuf,

    /// challenges import json
    #[clap(
        long = "challenges",
        default_value = "../resources/external/challenges.json"
    )]
    pub challenges: PathBuf,

    /// funds import json
    #[clap(long = "funds", default_value = "../resources/external/funds.json")]
    pub funds: PathBuf,

    /// reviews import json
    #[clap(long = "reviews", default_value = "../resources/external/reviews.json")]
    pub reviews: PathBuf,

    #[clap(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[clap(long = "skip-qr-generation")]
    pub skip_qr_generation: bool,

    #[clap(long = "log-level", default_value = "LogLevel::INFO")]
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
