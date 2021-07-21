mod burst;
mod constant;

use crate::load::{ArtificialUserLoad, MultiControllerError, NodeLoadError, ServicingStationLoad};
use burst::BurstIapyxLoadCommand;
use constant::ConstIapyxLoadCommand;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::Monitor;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxLoadCommandError {
    #[error("duration or requests per thread stategy has to be defined")]
    NoStrategyDefined,
    #[error("load runner error")]
    NodeLoadError(#[from] NodeLoadError),
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
    #[error("serialize error")]
    SerializeError(#[from] serde_json::Error),
    #[error("servicing station error")]
    ServicingStationError(#[from] crate::load::ServicingStationLoadError),
    #[error("artificial users error")]
    ArtificialUserError(#[from] crate::load::ArtificialUserLoadError),
}

#[derive(StructOpt, Debug)]
pub enum IapyxLoadCommand {
    NodeOnly(NodeOnlyLoadCommand),
    StaticOnly(StaticOnlyLoadCommand),
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

#[derive(StructOpt, Debug)]
pub struct ArtificialLoadCommand {
    #[structopt(short = "c", long = "config")]
    config: PathBuf,
}

impl ArtificialLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = serde_json::from_str(&jortestkit::file::read_file(&self.config))?;
        let load = ArtificialUserLoad::new(config);
        load.start().map(|_| ()).map_err(Into::into)
    }
}

#[derive(StructOpt, Debug)]
pub struct StaticOnlyLoadCommand {
    #[structopt(short = "c", long = "config")]
    config: PathBuf,
}

impl StaticOnlyLoadCommand {
    pub fn exec(&self) -> Result<(), IapyxLoadCommandError> {
        let config = serde_json::from_str(&jortestkit::file::read_file(&self.config))?;
        let load = ServicingStationLoad::new(config);
        load.start().map(|_| ()).map_err(Into::into)
    }
}

#[derive(StructOpt, Debug)]
pub enum NodeOnlyLoadCommand {
    Burst(BurstIapyxLoadCommand),
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
