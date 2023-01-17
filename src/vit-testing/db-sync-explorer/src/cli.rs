use clap::Parser;
use std::path::PathBuf;
use vit_servicing_station_lib::server::settings::LogLevel;

#[derive(Debug, Parser)]
#[non_exhaustive]
#[clap(about = "Create a voting power snapshot")]
/// CLI arguments for db-sync-explorer
pub struct Args {
    /// configuration file
    #[clap(long)]
    pub config: PathBuf,

    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(long = "log-level", default_value = "INFO")]
    pub log_level: LogLevel,
}
