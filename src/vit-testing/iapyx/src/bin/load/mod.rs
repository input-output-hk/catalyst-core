mod burst;
mod constant;

use burst::BurstIapyxLoadCommand;
use clap::Parser;
use constant::ConstIapyxLoadCommand;
use iapyx::ArtificialUserLoad;
use iapyx::MultiControllerError;
use iapyx::NodeLoadError;
use iapyx::ServicingStationLoad;
pub use jortestkit::console::progress_bar::ProgressBarMode;
use jortestkit::load::Monitor;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxLoadCommandError {
    #[error("load runner error")]
    NodeLoadError(#[from] NodeLoadError),
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
    #[error("serialize error")]
    SerializeError(#[from] serde_json::Error),
    #[error("servicing station error")]
    ServicingStationError(#[from] iapyx::ServicingStationLoadError),
    #[error("artificial users error")]
    ArtificialUserError(#[from] iapyx::ArtificialUserLoadError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Parser, Debug)]
pub enum IapyxLoadCommand {
    /// Load which targets blockchain calls only
    #[clap(subcommand)]
    NodeOnly(NodeOnlyLoadCommand),
    /// Load which targets static data only
    StaticOnly(StaticOnlyLoadCommand),
    /// Load with simulate real user case (both blockchain and static data in some relation)
    Simulation(ArtificialLoadCommand),
}

impl IapyxLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        match self {
            Self::NodeOnly(node_only) => node_only.exec(),
            Self::StaticOnly(static_only) => static_only.exec(),
            Self::Simulation(simulation) => simulation.exec(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct ArtificialLoadCommand {
    /// Path to configuration file
    #[clap(short = 'c', long = "config")]
    config: PathBuf,
}

impl ArtificialLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = serde_json::from_str(&jortestkit::file::read_file(&self.config)?)?;
        let load = ArtificialUserLoad::new(config);
        load.start().map(|_| ()).map_err(Into::into)
    }
}

#[derive(Parser, Debug)]
pub struct StaticOnlyLoadCommand {
    /// Path to configuration file
    #[clap(short = 'c', long = "config")]
    config: PathBuf,
}

impl StaticOnlyLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = serde_json::from_str(&jortestkit::file::read_file(&self.config)?)?;
        let load = ServicingStationLoad::new(config);
        load.start().map(|_| ()).map_err(Into::into)
    }
}

#[derive(Parser, Debug)]
pub enum NodeOnlyLoadCommand {
    /// Bursts mode. Sends votes in batches and then wait x seconds
    #[clap(subcommand)]
    Burst(BurstIapyxLoadCommand),
    /// Constant load. Sends votes with x votes per second speed.
    #[clap(subcommand)]
    Const(ConstIapyxLoadCommand),
}

impl NodeOnlyLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        match self {
            Self::Burst(burst) => burst.exec(),
            Self::Const(constant) => constant.exec(),
        }
    }
}

pub fn build_monitor(progress_bar_mode: &ProgressBarMode) -> Monitor {
    match progress_bar_mode {
        ProgressBarMode::Monitor => Monitor::Progress(100),
        ProgressBarMode::Standard => Monitor::Standard(100),
        ProgressBarMode::None => Monitor::Disabled(10),
    }
}
