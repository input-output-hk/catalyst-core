use crate::config::read_params;
use crate::mock::{read_config, start_rest_server, Configuration, Context};
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
    pub params: Option<PathBuf>,
}

impl MockStartCommandArgs {
    #[tokio::main]
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;
        let start_params = self.params.as_ref().map(|x| read_params(x).unwrap());

        if self.token.is_some() {
            configuration.token = self.token;
        }

        let control_context = Arc::new(Mutex::new(Context::new(
            configuration.clone(),
            start_params,
        )?));

        tokio::spawn(async move {
            start_rest_server(control_context.clone(), configuration).await;
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
    CannotReadConfiguration(#[from] crate::mock::MockConfigError),
    #[error(transparent)]
    CannotReadParameters(#[from] serde_yaml::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Mock(#[from] crate::mock::ContextError),
}
