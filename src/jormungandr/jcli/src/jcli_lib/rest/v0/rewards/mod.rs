mod epoch;
mod history;

use self::{epoch::Epoch, history::History};
use crate::jcli_lib::rest::Error;
use clap::Parser;

#[derive(Parser)]
#[clap(name = "rewards", rename_all = "kebab-case")]
pub enum Rewards {
    /// Rewards distribution history one or more epochs starting from the last one
    #[clap(subcommand)]
    History(History),
    /// Rewards distribution for a specific epoch
    #[clap(subcommand)]
    Epoch(Epoch),
}

impl Rewards {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Rewards::History(history) => history.exec(),
            Rewards::Epoch(epoch) => epoch.exec(),
        }
    }
}
