mod stats;

use self::stats::Stats;
use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Node {
    /// Node information
    #[clap(subcommand)]
    Stats(Stats),
}

impl Node {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Node::Stats(stats) => stats.exec(),
        }
    }
}
