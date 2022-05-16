mod count;
mod duration;

use crate::load::IapyxLoadCommandError;
pub use count::BurstCountIapyxLoadCommand;
pub use duration::BurstDurationIapyxLoadCommand;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum BurstIapyxLoadCommand {
    Duration(BurstDurationIapyxLoadCommand),
    Count(BurstCountIapyxLoadCommand),
}

impl BurstIapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        match self {
            Self::Duration(duration) => duration.exec(),
            Self::Count(count) => count.exec(),
        }
    }
}
