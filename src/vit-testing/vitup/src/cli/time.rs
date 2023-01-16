use crate::config::read_config;
use crate::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct TimeCommand {
    /// Path to configuration file
    #[clap(long = "config")]
    pub config: PathBuf,
}

impl TimeCommand {
    pub fn exec(self) -> Result<()> {
        std::env::set_var("RUST_BACKTRACE", "full");
        read_config(self.config)?.print_report(None);
        Ok(())
    }
}
