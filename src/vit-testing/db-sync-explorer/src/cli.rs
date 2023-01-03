use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[non_exhaustive]
#[clap(about = "Create a voting power snapshot")]
/// CLI arguments for db-sync-explorer
pub struct Args {
    /// configuration file
    #[clap(long)]
    pub config: PathBuf,
}
