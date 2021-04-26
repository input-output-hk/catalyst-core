mod count;
mod duration;

use crate::cli::args::load::IapyxLoadCommandError;
pub use count::ConstantCountIapyxLoadCommand;
pub use duration::ConstDurationIapyxLoadCommand;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ConstIapyxLoadCommand {
    Duration(ConstDurationIapyxLoadCommand),
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
