pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::Configuration;
use crate::job::JobOutputInfo;
use crate::request::Request;
use crate::rest::ServerStopper;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

pub struct Context {
    server_stopper: Option<ServerStopper>,
    config: Configuration,
    address: SocketAddr,
    state: State,
}

impl Context {
    pub fn new(config: Configuration) -> Self {
        Self {
            server_stopper: None,
            address: ([0, 0, 0, 0], config.port).into(),
            config,
            state: State::Idle,
        }
    }

    pub fn set_server_stopper(&mut self, server_stopper: ServerStopper) {
        self.server_stopper = Some(server_stopper)
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        &self.server_stopper
    }

    pub fn new_run(&mut self, request: Request) -> Result<Uuid, Error> {
        match self.state {
            State::Idle | State::Finished { .. } => {
                let id = Uuid::new_v4();
                self.state = State::RequestToStart {
                    job_id: id,
                    request,
                };
                Ok(id)
            }
            _ => Err(Error::RegistrationInProgress),
        }
    }

    pub fn run_started(&mut self) -> Result<(), Error> {
        match &self.state {
            State::RequestToStart { job_id, request } => {
                self.state = State::Running {
                    job_id: *job_id,
                    start: Utc::now().naive_utc(),
                    request: request.clone(),
                    step: Step::RunningSnapshot,
                };
                Ok(())
            }
            _ => Err(Error::NoRequestToStart),
        }
    }

    pub fn run_finished(&mut self, info: JobOutputInfo) -> Result<(), Error> {
        println!("{:?}", self.state);

        match &self.state {
            State::Running {
                job_id,
                start,
                request,
                step,
            } => {
                self.state = State::Finished {
                    job_id: *job_id,
                    start: *start,
                    end: Utc::now().naive_utc(),
                    request: request.clone(),
                    info,
                };
                Ok(())
            }
            _ => Err(Error::RegistrationNotStarted),
        }
    }

    pub fn status_by_id(&self, id: Uuid) -> Result<State, Error> {
        match self.state {
            State::Idle => Err(Error::NoJobRun),
            State::RequestToStart { .. } => Ok(self.state.clone()),
            State::Running { job_id, .. } => {
                if job_id == id {
                    Ok(self.state.clone())
                } else {
                    Err(Error::JobNotFound)
                }
            }
            State::Finished { job_id, .. } => {
                if job_id == id {
                    Ok(self.state.clone())
                } else {
                    Err(Error::JobNotFound)
                }
            }
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut State {
        &mut self.state
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn config(&self) -> &Configuration {
        &self.config
    }

    pub fn api_token(&self) -> Option<String> {
        self.config.token.clone()
    }

    pub fn set_api_token(&mut self, api_token: String) {
        self.config.token = Some(api_token);
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum State {
    Idle,
    RequestToStart {
        job_id: Uuid,
        request: Request,
    },
    Running {
        job_id: Uuid,
        start: NaiveDateTime,
        request: Request,
        step: Step,
    },
    Finished {
        job_id: Uuid,
        start: NaiveDateTime,
        end: NaiveDateTime,
        request: Request,
        info: JobOutputInfo,
    },
}

impl State {
    pub fn update_running_step(&mut self, new_step: Step) {
        if let State::Running { step, .. } = self {
            *step = new_step;
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub enum Step {
    RunningSnapshot,
    ExtractingQRCode,
    VerifyingRegistration,
}

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
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
