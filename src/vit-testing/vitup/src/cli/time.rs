use crate::builders::utils::io::read_config;
use crate::builders::VitBackendSettingsBuilder;
use crate::error::Result;
use std::path::PathBuf;
use structopt::StructOpt;
#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct TimeCommand {
    /// configuration
    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl TimeCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");

        let config = read_config(&self.config)?;

        let mut quick_setup = VitBackendSettingsBuilder::new();
        quick_setup.skip_qr_generation();
        quick_setup.upload_parameters(config.params);
        quick_setup.print_report();
        Ok(())
    }
}
