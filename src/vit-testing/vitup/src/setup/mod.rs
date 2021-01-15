pub mod quick;

use crate::error::Result;
use crate::setup::quick::QuickStartCommandArgs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum VitCliCommand {
    /// start backend
    Start(StartCommand),
}

impl VitCliCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum StartCommand {
    /// start backend from scratch
    Quick(QuickStartCommandArgs),
}

impl StartCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Quick(quick_start_command) => quick_start_command.exec(),
        }
    }
}
