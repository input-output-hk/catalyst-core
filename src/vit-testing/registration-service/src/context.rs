pub type ContextLock = Arc<Mutex<Context>>;
pub type State = scheduler_service_lib::State<Request, (), JobOutputInfo>;

use crate::cardano::cli::CardanoCli;
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

    pub fn cardano_cli_executor(&self) -> CardanoCli {
        CardanoCli::new(self.config.cardano_cli.clone())
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

    pub fn api_token(&self) -> Option<String> {
        self.config.inner.api_token.clone()
    }

    pub fn set_api_token(&mut self, api_token: String) {
        self.config.inner.api_token = Some(api_token);
    }

    pub fn slot_no(&self) -> Result<u64, Error> {
        match &self.state {
            State::Finished { info, .. } => Ok(info
                .as_ref()
                .ok_or(Error::CannotGetSlotNoFromRegistrationResult)?
                .slot_no),
            _ => Err(Error::CannotGetSlotNoFromRegistrationResult),
        }
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }
}

impl Context {
    pub fn into_scheduler_context(&self) -> SchedulerContext {
        self.scheduler_context.clone()
    }
}

use scheduler_service_lib::{SchedulerContext, ServerStopper};
use thiserror::Error;

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
    #[error("cannot get slot no from registration state")]
    CannotGetSlotNoFromRegistrationResult,
}
