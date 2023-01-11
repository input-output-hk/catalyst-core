use crate::config::read_config;
use std::path::PathBuf;
use clap::Parser;
use thiserror::Error;

#[derive(Parser, Debug)]
pub struct ConfigValidateCommand {
    /// target config
    #[clap(name = "CONFIG")]
    pub config: PathBuf,
}

impl ConfigValidateCommand {
    pub fn exec(self) -> Result<(), Error> {
        read_config(self.config)
            .map(|_| ())
            .map_err(|e| Error::ValidationError(e.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("validation error: '{0}'")]
    ValidationError(String),
}
