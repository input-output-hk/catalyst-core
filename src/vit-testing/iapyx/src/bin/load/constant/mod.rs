mod count;
mod duration;

use crate::load::IapyxLoadCommandError;
pub use count::ConstantCountIapyxLoadCommand;
pub use duration::ConstDurationIapyxLoadCommand;
use clap::Parser;

#[derive(Parser, Debug)]
pub enum ConstIapyxLoadCommand {
    /// Duration based load. Defines how much time load should run
    Duration(ConstDurationIapyxLoadCommand),
    /// Requests count based load. Defines how many requests load should sent in total
    Count(ConstantCountIapyxLoadCommand),
}

impl ConstIapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        match self {
            Self::Duration(duration) => duration.exec(),
            Self::Count(count) => count.exec(),
        }
    }
}
