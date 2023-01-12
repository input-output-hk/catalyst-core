mod config;
mod deployment;
mod ideascale;

use clap::Parser;
use config::ConfigValidateCommand;
use deployment::CheckError;
use deployment::DeploymentValidateCommand;
pub use ideascale::Error as IdeascaleError;
use ideascale::IdeascaleValidateCommand;
use thiserror::Error;

#[derive(Parser, Debug)]
pub enum ValidateCommand {
    Ideascale(IdeascaleValidateCommand),
    Deployment(DeploymentValidateCommand),
    Config(ConfigValidateCommand),
}

impl ValidateCommand {
    pub fn exec(self) -> Result<(), Error> {
        match self {
            Self::Ideascale(ideascale) => ideascale.exec().map_err(Into::into),
            Self::Deployment(deployment) => deployment.exec().map_err(Into::into),
            Self::Config(config) => config.exec().map_err(Into::into),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] config::Error),
    #[error(transparent)]
    Deployment(#[from] CheckError),
    #[error(transparent)]
    Ideascale(#[from] ideascale::Error),
}
