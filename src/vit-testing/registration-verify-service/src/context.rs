pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::Configuration;
use crate::job::JobOutputInfo;
use crate::request::Request;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Context {
    scheduler_context: SchedulerContext,
    config: Configuration,
    state: State,
}

impl RunContext<Request, JobOutputInfo> for Context {
    fn run_requested(&self) -> Option<(Uuid, Request)> {
        self.state.run_requested()
    }

    fn new_run_started(&mut self) -> Result<(), scheduler_service_lib::Error> {
        self.state.new_run_started()
    }

    fn run_finished(
        &mut self,
        output_info: Option<JobOutputInfo>,
    ) -> Result<(), scheduler_service_lib::Error> {
        self.state.run_finished(output_info)
    }
}

impl Context {
    pub fn new(config: Configuration) -> Self {
        Self {
            scheduler_context: SchedulerContext::new(None, config.inner.clone()),
            config,
            state: State::Idle,
        }
    }

    pub fn set_server_stopper(&mut self, server_stopper: ServerStopper) {
        self.scheduler_context
            .set_server_stopper(Some(server_stopper));
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        self.scheduler_context.server_stopper()
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn address(&self) -> &SocketAddr {
        &self.config.inner.address
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Configuration {
        &mut self.config
    }

    pub fn set_snapshot_job_id(&mut self, snapshot_job_id: String) {
        self.config.snapshot_job_id = Some(snapshot_job_id);
    }
}

impl Context {
    pub fn into_scheduler_context(&self) -> SchedulerContext {
        self.scheduler_context.clone()
    }
}

pub type State = scheduler_service_lib::State<Request, Step, JobOutputInfo>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Step {
    RunningSnapshot,
    BuildingAddress,
    VerifyingRegistration,
}

use scheduler_service_lib::{RunContext, SchedulerContext, ServerStopper};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("job is in progress.")]
    RegistrationInProgress,
    #[error("job hasn't been started")]
    RegistrationNotStarted,
    #[error("no request to start")]
    NoRequestToStart,
    #[error("job was not found")]
    JobNotFound,
    #[error("no job was run yet")]
    NoJobRun,
}
