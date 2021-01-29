pub mod initials;
pub mod qr;
pub mod quick;

use crate::error::Result;
use crate::setup::qr::QrCommandArgs;
use crate::setup::quick::QuickStartCommandArgs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum VitCliCommand {
    /// start backend
    Start(StartCommand),
    /// start backend
    Generate(GenerateCommand),
}

impl VitCliCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec(),
            Self::Generate(generate_command) => generate_command.exec(),
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

#[derive(StructOpt, Debug)]
pub enum GenerateCommand {
    /// generate qrs
    Qr(QrCommandArgs),
}

impl GenerateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Qr(quick_start_command) => quick_start_command.exec(),
        }
    }
}
