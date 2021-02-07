pub mod generate;
pub mod start;

use crate::error::Result;
use crate::setup::generate::QrCommandArgs;
use generate::DataCommandArgs;
use start::QuickStartCommandArgs;
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
    /// generate data only
    Data(DataCommandArgs),
}

impl GenerateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Qr(quick_start_command) => quick_start_command.exec(),
            Self::Data(data_start_command) => data_start_command.exec(),
        }
    }
}
