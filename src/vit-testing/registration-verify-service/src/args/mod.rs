use crate::{
    config::{read_config, Configuration},
    context::{Context, ContextLock},
};

use crate::job::RegistrationVerifyJobBuilder;
use crate::rest::start_rest_server;
use scheduler_service_lib::{spawn_scheduler, ManagerService, WrappedPoisonError};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use thiserror::Error;

#[derive(StructOpt, Debug)]
pub struct RegistrationVerifyServiceCommand {
    #[structopt(long = "api-token")]
    pub api_token: Option<String>,

    #[structopt(long = "admin-token")]
    pub admin_token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl RegistrationVerifyServiceCommand {
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.api_token.is_some() {
            configuration.inner.api_token = self.api_token;
        }

        if self.admin_token.is_some() {
            configuration.inner.admin_token = self.admin_token;
        }

        let control_context: ContextLock =
            Arc::new(Mutex::new(Context::new(configuration.clone())));

        let mut manager = ManagerService::default();
        let handle = manager.spawn(start_rest_server(control_context.clone()));

        let job = RegistrationVerifyJobBuilder::new()
            .with_jcli(&configuration.jcli)
            .with_snapshot_token(&configuration.snapshot_token)
            .with_snapshot_address(&configuration.snapshot_address)
            .with_network(configuration.network)
            .build();

        spawn_scheduler(
            &configuration.inner.result_dir,
            control_context,
            Box::new(job),
            handle,
        )
        .await
        .map_err(Into::into)
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
    #[error(transparent)]
    Job(#[from] crate::job::Error),
    #[error(transparent)]
    Scheduler(#[from] scheduler_service_lib::Error),
    #[error(transparent)]
    Poison(#[from] WrappedPoisonError),
}
