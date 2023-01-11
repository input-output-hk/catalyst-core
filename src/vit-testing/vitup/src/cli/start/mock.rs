use crate::builders::utils::logger;
use crate::mode::mock::{farm, read_config, start_rest_server, Configuration, Context};
use jormungandr_automation::jormungandr::LogLevel;
use std::sync::{Mutex, RwLock};
use std::{path::PathBuf, sync::Arc};
use clap::Parser;
use thiserror::Error;
use tracing::subscriber::SetGlobalDefaultError;

#[derive(Parser, Debug)]
pub struct MockStartCommandArgs {
    #[clap(long = "token")]
    pub token: Option<String>,

    #[clap(long = "config")]
    pub config: PathBuf,

    #[clap(long = "params")]
    pub params: Option<PathBuf>,

    #[clap(long = "log-level", default_value = "INFO")]
    pub log_level: LogLevel,
}

impl MockStartCommandArgs {
    #[tokio::main]
    pub async fn exec(self) -> Result<(), Error> {
        logger::init(self.log_level)?;

        let mut configuration: Configuration = read_config(&self.config)?;
        let start_params = self
            .params
            .as_ref()
            .map(|x| crate::config::read_config(x).unwrap());

        if self.token.is_some() {
            configuration.token = self.token;
        }

        let context = Context::new(configuration, start_params)?;
        let control_context = Arc::new(RwLock::new(context));

        tokio::spawn(async move { start_rest_server(control_context.clone()).await.unwrap() })
            .await
            .map(|_| ())
            .map_err(Into::into)
    }
}

#[derive(Parser, Debug)]
#[clap(setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct MockFarmCommand {
    /// path to config file
    #[clap(long = "config", short = "c")]
    pub config: PathBuf,
}

impl MockFarmCommand {
    #[tokio::main]
    pub async fn exec(self) -> Result<(), Error> {
        let control_context = Arc::new(Mutex::new(farm::Context::new(
            farm::read_config(&self.config).unwrap(),
        )));
        tokio::spawn(async move {
            farm::start_rest_server(control_context.clone())
                .await
                .unwrap()
        })
        .await
        .map(|_| ())
        .map_err(Into::into)
    }
}

#[derive(Debug, Error)]
#[allow(clippy::large_enum_variant)]
pub enum Error {
    #[error(transparent)]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error(transparent)]
    CannotReadConfiguration(#[from] crate::mode::mock::MockConfigError),
    #[error(transparent)]
    CannotReadParameters(#[from] serde_yaml::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Mock(#[from] crate::mode::mock::ContextError),
    #[error(transparent)]
    Farm(#[from] crate::mode::mock::farm::ContextError),
    #[error(transparent)]
    ServerError(#[from] crate::mode::mock::RestError),
    #[error(transparent)]
    SetGlobalDefault(#[from] SetGlobalDefaultError),
}
