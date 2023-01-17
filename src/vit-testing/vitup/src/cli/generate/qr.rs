use crate::builders::utils::SessionSettingsExtension;
use crate::builders::utils::{logger, DeploymentTree};
use crate::builders::VitBackendSettingsBuilder;
use crate::config::ConfigBuilder;
use crate::config::Initials;
use crate::Result;
use clap::Parser;
use hersir::config::SessionSettings;
use jormungandr_automation::jormungandr::LogLevel;
use jortestkit::prelude::read_file;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct QrCommandArgs {
    /// Careful! directory would be removed before export
    #[clap(long = "root-dir", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[clap(long = "count")]
    pub initials: Option<usize>,

    #[clap(long = "initials", conflicts_with = "count")]
    pub initials_mapping: Option<PathBuf>,

    #[clap(long = "global-pin", default_value = "1234")]
    pub global_pin: String,

    #[clap(long = "log-level", default_value = "LogLevel::INFO")]
    pub log_level: LogLevel,
}

impl QrCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        logger::init(self.log_level)?;

        let session_settings = SessionSettings::from_dir(&self.output_directory);

        let mut config_builder = ConfigBuilder::default();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping)?;
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            config_builder = config_builder.initials(initials);
        } else {
            config_builder =
                config_builder.block0_initials_count(self.initials.unwrap(), &self.global_pin);
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        } else {
            std::fs::remove_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory);
        let config = config_builder.build();

        println!("{:?}", config.initials);
        let _ = VitBackendSettingsBuilder::default()
            .session_settings(session_settings)
            .config(&config)
            .build()?;

        //remove block0.bin
        std::fs::remove_file(deployment_tree.block0_path())?;

        println!("Qrs dumped into {:?}", self.output_directory);
        Ok(())
    }
}
