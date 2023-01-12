pub mod diff;
pub mod generate;
pub mod import;
pub mod start;
pub mod time;
pub mod validate;

use self::time::TimeCommand;
use crate::cli::generate::{CommitteeIdCommandArgs, QrCommandArgs, SnapshotCommandArgs};
use crate::cli::start::AdvancedStartCommandArgs;
use crate::cli::start::{MockFarmCommand, MockStartCommandArgs};
use crate::Result;
use clap::Parser;
use diff::DiffCommand;
use generate::DataCommandArgs;
use import::ImportCommand;
use start::QuickStartCommandArgs;
pub use validate::Error as ValidateError;
use validate::ValidateCommand;

#[allow(clippy::large_enum_variant)]
#[derive(Parser, Debug)]
pub enum VitCliCommand {
    /// Starts catalyst backend
    #[clap(subcommand)]
    Start(StartCommand),
    /// Generates fund data
    #[clap(subcommand)]
    Generate(GenerateCommand),
    /// Prints differences between new deployment and target env
    Diff(DiffCommand),
    /// Validates static ideascale data
    #[clap(subcommand)]
    Validate(ValidateCommand),
    /// Import data
    #[clap(subcommand)]
    Import(ImportCommand),
    /// Convert time defined in config to UTC
    Time(TimeCommand),
}

impl VitCliCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Start(start_command) => start_command.exec(),
            Self::Generate(generate_command) => generate_command.exec(),
            Self::Diff(diff_command) => diff_command.exec(),
            Self::Validate(validate_command) => validate_command.exec().map_err(Into::into),
            Self::Import(import_command) => import_command.exec().map_err(Into::into),
            Self::Time(time_command) => time_command.exec(),
        }
    }
}

#[derive(Parser, Debug)]
pub enum StartCommand {
    /// start backend from scratch
    Quick(QuickStartCommandArgs),
    /// start advanced backend from scratch
    Advanced(AdvancedStartCommandArgs),
    /// start mock env
    Mock(MockStartCommandArgs),
    /// start multiple mock environments
    MockFarm(MockFarmCommand),
}

impl StartCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Quick(quick_start_command) => quick_start_command.exec(),
            Self::Advanced(advanced_start_command) => advanced_start_command.exec(),
            Self::Mock(mock_start_command) => mock_start_command.exec().map_err(Into::into),
            Self::MockFarm(mock_farm_start_command) => {
                mock_farm_start_command.exec().map_err(Into::into)
            }
        }
    }
}

#[derive(Parser, Debug)]
pub enum GenerateCommand {
    /// generate qrs
    Qr(QrCommandArgs),
    /// generate data only
    #[clap(subcommand)]
    Data(DataCommandArgs),
    /// generate snapshot data only
    Snapshot(SnapshotCommandArgs),
    /// Committee Id
    Committee(CommitteeIdCommandArgs),
}

impl GenerateCommand {
    pub fn exec(self) -> Result<()> {
        match self {
            Self::Qr(quick_start_command) => quick_start_command.exec(),
            Self::Data(data_start_command) => data_start_command.exec(),
            Self::Snapshot(snapshot_start_command) => snapshot_start_command.exec(),
            Self::Committee(generate_committee_command) => generate_committee_command.exec(),
        }
    }
}
