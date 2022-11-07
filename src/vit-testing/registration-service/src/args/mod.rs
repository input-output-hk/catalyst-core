use crate::job::{JobOutputInfo, VoteRegistrationJobBuilder};
use crate::request::Request;
use crate::{
    config::{read_config, Configuration},
    context::Context,
    start_rest_server,
};
use scheduler_service_lib::{spawn_scheduler, ManagerService, RunContext};
use std::sync::Mutex;
use std::{path::PathBuf, sync::Arc};
use structopt::StructOpt;
use thiserror::Error;
use uuid::Uuid;

#[derive(StructOpt, Debug)]
pub struct RegistrationServiceCommand {
    #[structopt(long = "token")]
    pub token: Option<String>,

    #[structopt(long = "config")]
    pub config: PathBuf,
}

impl RunContext<Request, JobOutputInfo> for Context {
    fn run_requested(&self) -> Option<(Uuid, Request)> {
        self.state().run_requested()
    }

    fn new_run_started(&mut self) -> Result<(), scheduler_service_lib::Error> {
        self.state_mut().new_run_started()
    }

    fn run_finished(
        &mut self,
        output_info: Option<JobOutputInfo>,
    ) -> Result<(), scheduler_service_lib::Error> {
        self.state_mut().run_finished(output_info)
    }
}

impl RegistrationServiceCommand {
    pub async fn exec(self) -> Result<(), Error> {
        let mut configuration: Configuration = read_config(&self.config)?;

        if self.token.is_some() {
            configuration.inner.api_token = self.token;
        }

        let control_context = Arc::new(Mutex::new(Context::new(configuration.clone())));

        let mut manager = ManagerService::default();
        let handle = manager.spawn(start_rest_server(control_context.clone()));
        let job = VoteRegistrationJobBuilder::new(configuration.clone()).build();
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
    Job(#[from] crate::Error),
    #[error(transparent)]
    Scheduler(#[from] scheduler_service_lib::Error),
    #[error(transparent)]
    Inner(#[from] scheduler_service_lib::CliError),
}
