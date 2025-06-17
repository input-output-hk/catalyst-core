use crate::job::SnapshotJobRunner;
use crate::rest::start_rest_server;
use crate::{
    config::{read_config, Configuration},
    Context,
};
use clap::Parser;
use scheduler_service_lib::{spawn_scheduler, ManagerService, WrappedPoisonError};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use thiserror::Error;

#[derive(Parser, Debug)]
pub struct TriggerServiceCommand {
    #[clap(long = "token")]
    pub token: Option<String>,
    #[clap(long = "config")]
    pub config: PathBuf,
}

impl TriggerServiceCommand {
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.token.is_some() {
            configuration.set_token(self.token);
        }

        let control_context = Arc::new(Mutex::new(Context::new(configuration.clone())));

        let mut manager = ManagerService::default();
        let handle = manager.spawn(start_rest_server(control_context.clone()));
        let job_runner = SnapshotJobRunner(configuration.clone());

        spawn_scheduler(
            configuration.result_dir(),
            control_context,
            Box::new(job_runner),
            handle,
        )
        .await}
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot spawn configuration")]
    CannotSpawnCommand(#[from] std::io::Error),
    #[error("cannot read configuration")]
    CannotReadConfiguration(#[from] crate::config::Error),
    #[error("context error")]
    Context(#[from] crate::context::Error),
    #[error("cannot persist job state")]
    CannotPersistJobState,
    #[error("cannot serialize job state")]
    CannotSerializeJobState(#[from] serde_yaml::Error),
    #[error(transparent)]
    Scheduler(#[from] scheduler_service_lib::Error),
    #[error(transparent)]
    Poison(#[from] WrappedPoisonError),
}
