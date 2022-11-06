pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::Configuration;
use crate::config::JobParameters;
use scheduler_service_lib::{RunContext, SchedulerContext, ServerStopper, State};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub type ContextState = State<JobParameters, (), ()>;

pub struct Context {
    inner: SchedulerContext,
    state: ContextState,
}

impl RunContext<JobParameters, ()> for Context {
    fn run_requested(&self) -> Option<(Uuid, JobParameters)> {
        self.state.run_requested()
    }

    fn new_run_started(&mut self) -> Result<(), scheduler_service_lib::Error> {
        self.state.new_run_started()
    }

    fn run_finished(
        &mut self,
        output_info: Option<()>,
    ) -> Result<(), scheduler_service_lib::Error> {
        self.state.run_finished(output_info)
    }
}

impl Context {
    pub fn new(config: Configuration) -> Self {
        Self {
            inner: SchedulerContext::new(None, config.inner),
            state: ContextState::Idle,
        }
    }

    pub fn set_server_stopper(&mut self, server_stopper: ServerStopper) {
        self.inner.set_server_stopper(Some(server_stopper));
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        self.inner.server_stopper()
    }

    pub fn state(&self) -> &ContextState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ContextState {
        &mut self.state
    }

    pub fn address(&self) -> &SocketAddr {
        &self.inner.config().address
    }

    pub fn api_token(&self) -> Option<String> {
        self.inner.config().api_token.clone()
    }

    pub fn set_api_token(&mut self, api_token: String) {
        self.inner.set_api_token(Some(api_token));
    }

    pub fn working_directory(&self) -> &Option<PathBuf> {
        self.inner.working_directory()
    }
}

impl Context {
    pub fn into_scheduler_context(&self) -> SchedulerContext {
        self.inner.clone()
    }
}

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("job is in progress.")]
    SnaphotInProgress,
    #[error("job hasn't been started")]
    SnaphotNotStarted,
    #[error("no request to start")]
    NoRequestToStart,
    #[error("job was not found")]
    JobNotFound,
    #[error("no job was run yet")]
    NoJobRun,
}
