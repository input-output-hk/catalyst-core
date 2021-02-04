use crate::config::Initials;
use crate::setup::start::QuickVitBackendSettingsBuilder;
use crate::Result;
use jormungandr_scenario_tests::ProgressBarMode as ScenarioProgressBarMode;
use jormungandr_scenario_tests::{Context, Seed};
use jortestkit::prelude::read_file;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct QrCommandArgs {
    /// Careful! directory would be removed before export
    #[structopt(long = "root-dir", default_value = "./data")]
    pub output_directory: PathBuf,

    /// how many qr to generate
    #[structopt(long = "count")]
    pub initials: Option<usize>,

    #[structopt(long = "initials")]
    pub initials_mapping: Option<PathBuf>,

    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,
}

impl QrCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let context = Context::new(
            Seed::generate(rand::rngs::OsRng),
            PathBuf::new(),
            PathBuf::new(),
            Some(self.output_directory.clone()),
            false,
            ScenarioProgressBarMode::None,
            "info".to_string(),
        );

        let mut quick_setup = QuickVitBackendSettingsBuilder::new();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping);
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            quick_setup.initials(initials);
        } else if let Some(initials_count) = self.initials {
            quick_setup.initials_count(initials_count, &self.global_pin);
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        }

        println!("{:?}", quick_setup.parameters().initials);
        quick_setup.build(context)?;

        println!("Qrs dumped into {:?}", self.output_directory);
        Ok(())
    }
}
