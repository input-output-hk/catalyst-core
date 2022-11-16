use crate::builders::utils::SessionSettingsExtension;
use crate::builders::utils::{logger, DeploymentTree};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::read_config;
use crate::mode::standard::generate_random_database;
use crate::Result;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct AllRandomDataCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "output", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[structopt(long = "config")]
    pub config: PathBuf,

    #[structopt(long = "snapshot")]
    pub snapshot: Option<PathBuf>,

    #[structopt(long = "log-level", default_value = "LogLevel::Info")]
    pub log_level: LogLevel,
}

impl AllRandomDataCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        logger::init(self.log_level)?;

        let session_settings = SessionSettings::from_dir(&self.output_directory);

        let mut config = read_config(&self.config)?;

        if let Some(snapshot) = self.snapshot {
            config.extend_from_initials_file(snapshot, chain_addr::Discrimination::Production)?;
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory);

        let (controller, vit_parameters) = VitBackendSettingsBuilder::default()
            .config(&config)
            .session_settings(session_settings)
            .build()?;

        generate_random_database(&deployment_tree, vit_parameters)?;

        config.print_report(Some(controller));
        Ok(())
    }
}
