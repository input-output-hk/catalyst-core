use crate::builders::post_deployment::DeploymentTree;
use crate::builders::utils::SessionSettingsExtension;
use crate::builders::VitBackendSettingsBuilder;
use crate::config::Initials;
use crate::Result;
use hersir::config::SessionSettings;
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

    #[structopt(long = "initials", conflicts_with = "count")]
    pub initials_mapping: Option<PathBuf>,

    #[structopt(long = "global-pin", default_value = "1234")]
    pub global_pin: String,
}

impl QrCommandArgs {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let session_settings = SessionSettings::empty_from_dir(&self.output_directory);

        let mut quick_setup = VitBackendSettingsBuilder::new();

        if let Some(mapping) = self.initials_mapping {
            let content = read_file(mapping);
            let initials: Initials =
                serde_json::from_str(&content).expect("JSON was not well-formatted");
            quick_setup.initials(initials);
        } else {
            quick_setup.initials_count(self.initials.unwrap(), &self.global_pin);
        }

        if !self.output_directory.exists() {
            std::fs::create_dir_all(&self.output_directory)?;
        } else {
            std::fs::remove_dir_all(&self.output_directory)?;
        }

        let deployment_tree = DeploymentTree::new(&self.output_directory, quick_setup.title());

        println!("{:?}", quick_setup.parameters().initials);
        quick_setup.build(session_settings.into())?;

        //remove block0.bin
        std::fs::remove_file(deployment_tree.block0_path())?;

        println!("Qrs dumped into {:?}", self.output_directory);
        Ok(())
    }
}
