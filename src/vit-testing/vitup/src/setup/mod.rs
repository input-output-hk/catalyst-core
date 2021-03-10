pub mod generate;
pub mod start;

use crate::error::Result;
use crate::mock::MockStartCommandArgs;
use crate::setup::generate::{QrCommandArgs, SnapshotCommandArgs};
use crate::setup::start::AdvancedStartCommandArgs;
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
    pub async fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec().await,
            Self::Generate(generate_command) => generate_command.exec(),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum StartCommand {
    /// start backend from scratch
    Quick(QuickStartCommandArgs),
    /// start advanced backend from scratch
    Advanced(AdvancedStartCommandArgs),
    // start mock env
    Mock(MockStartCommandArgs),
}

impl StartCommand {
    pub async fn exec(self) -> Result<()> {
        match self {
            Self::Quick(quick_start_command) => quick_start_command.exec(),
            Self::Advanced(advanced_start_command) => advanced_start_command.exec(),
            Self::Mock(mock_start_command) => mock_start_command.exec().await.map_err(Into::into),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum GenerateCommand {
    /// generate qrs
    Qr(QrCommandArgs),
    /// generate data only
    Data(DataCommandArgs),
    /// generate snapshot data only
    Snapshot(SnapshotCommandArgs),
}

impl GenerateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Qr(quick_start_command) => quick_start_command.exec(),
            Self::Data(data_start_command) => data_start_command.exec(),
            Self::Snapshot(snapshot_start_command) => snapshot_start_command.exec(),
        }
    }
}
