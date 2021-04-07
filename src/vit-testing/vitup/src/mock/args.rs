use crate::config::VitStartParameters;
use crate::mock::rest::start_rest_server;
use crate::mock::{
    config::{read_config, Configuration},
    context::Context,
};
use std::path::Path;
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct MockStartCommandArgs {
    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,

    #[structopt(long = "params")]
    pub params: PathBuf,
}

impl MockStartCommandArgs {
    pub fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;
        let start_params = read_params(&self.params)?;

        if self.token.is_some() {
            configuration.token = self.token;
        }

        let control_context = Arc::new(Mutex::new(Context::new(
            configuration.clone(),
            start_params,
        )));

        tokio::spawn(async move {
            start_rest_server(control_context.clone())
        });
        Ok(())
    }
}

pub fn read_params<P: AsRef<Path>>(params: P) -> Result<VitStartParameters, Error> {
    let contents = std::fs::read_to_string(&params)?;
    serde_yaml::from_str(&contents).map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::mock::config::Error),
    #[error("cannot read parameters")]
    CannotReadParameters(#[from] serde_yaml::Error),
    #[error("context error")]
    Context(#[from] crate::mock::context::Error),
}
