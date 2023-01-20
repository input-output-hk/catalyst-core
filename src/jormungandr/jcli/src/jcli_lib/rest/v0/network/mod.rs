mod stats;

use self::stats::Stats;
use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(rename_all = "kebab-case")]
pub enum Network {
    /// Network information
    #[clap(subcommand)]
    Stats(Stats),
}

impl Network {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Network::Stats(stats) => stats.exec(),
        }
    }
}
