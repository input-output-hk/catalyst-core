use crate::config::read_config;
use crate::Result;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct TimeCommand {
    /// Path to configuration file
    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl TimeCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");
        read_config(&self.config)?.print_report(None);
        Ok(())
    }
}
