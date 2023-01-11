use std::path::PathBuf;
use clap::Parser;

///
/// Hersir is a command line tool that lets you deploy a network of Jormungandr nodes
///
#[derive(Parser)]
pub struct Args {
    /// Path to config file
    #[clap(long, short)]
    pub config: PathBuf,

    /// Enable verbose mode
    #[clap(long, short)]
    pub verbose: bool,
}
