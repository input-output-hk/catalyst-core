use crate::mjolnir_lib::MjolnirError;
use clap::Parser;
use thiserror::Error;

mod batch;
mod standard;

#[derive(Parser, Debug)]
pub enum FragmentLoadCommand {
    /// sends fragments using batch endpoint
    #[clap(subcommand)]
    Batch(batch::Batch),
    /// sends fragments in single manner
    #[clap(subcommand)]
    Standard(standard::Standard),
}

#[derive(Error, Debug)]
pub enum FragmentLoadCommandError {
    #[error("Client Error")]
    ClientError(#[from] Box<MjolnirError>),
}

impl FragmentLoadCommand {
    pub fn exec(&self) -> Result<(), MjolnirError> {
        match self {
            FragmentLoadCommand::Batch(batch) => batch.exec(),
            FragmentLoadCommand::Standard(standard) => standard.exec(),
        }
    }
}
