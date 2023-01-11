mod count;
mod duration;

use crate::load::IapyxLoadCommandError;
pub use count::BurstCountIapyxLoadCommand;
pub use duration::BurstDurationIapyxLoadCommand;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum BurstIapyxLoadCommand {
    /// Duration based load. Defines how much time load should run
    Duration(BurstDurationIapyxLoadCommand),
    /// Requests count based load. Defines how many requests load should sent in total
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
