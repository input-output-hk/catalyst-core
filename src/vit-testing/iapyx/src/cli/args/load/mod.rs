mod burst;
mod constant;

use crate::load::IapyxLoadError;
use crate::load::MultiControllerError;
use burst::BurstIapyxLoadCommand;
use constant::ConstIapyxLoadCommand;
pub use jortestkit::console::progress_bar::{parse_progress_bar_mode_from_str, ProgressBarMode};
use jortestkit::load::Monitor;
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IapyxLoadCommandError {
    #[error("duration or requests per thread stategy has to be defined")]
    NoStrategyDefined,
    #[error("load runner error")]
    IapyxLoadError(#[from] IapyxLoadError),
    #[error("internal error")]
    MultiControllerError(#[from] MultiControllerError),
}

#[derive(StructOpt, Debug)]
pub enum IapyxLoadCommand {
    Burst(BurstIapyxLoadCommand),
    Const(ConstIapyxLoadCommand),
}

impl IapyxLoadCommand {
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
