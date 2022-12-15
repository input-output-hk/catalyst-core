mod client;
mod config;
mod context;
mod file_lister;
pub mod rest;
mod service;
mod state;

pub use client::{
    rest::Error as RestError, CliError, FilesCommand, HealthCommand, SchedulerRestClient,
    StatusCommand,
};
pub use config::Configuration;
pub use context::SchedulerContext;
pub use file_lister::{dump_json, Error as FileListerError, FolderDump};
use futures::channel::mpsc;
pub use service::{spawn_scheduler, ManagerService, WrappedPoisonError};
pub use state::State;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub trait RunContext<JobRequest, JobOutputInfo> {
    fn run_requested(&self) -> Option<(Uuid, JobRequest)>;
    fn new_run_started(&mut self) -> Result<(), crate::Error>;
    fn run_finished(&mut self, output_info: Option<JobOutputInfo>) -> Result<(), crate::Error>;
}

pub trait JobRunner<JobRequest, JobOutputInfo, Error> {
    fn start(
        &self,
        request: JobRequest,
        working_dir: PathBuf,
    ) -> Result<Option<JobOutputInfo>, Error>;
}

use thiserror::Error;
use uuid::Uuid;
use warp::reject::Reject;

impl Reject for Error {}

#[derive(Debug, Error, Deserialize, Serialize)]
pub enum Error {
    #[error("no request to start")]
    NoRequestToStart,
    #[error("job was not started")]
    JobNotStarted,
    #[error("job already in progress ")]
    JobInProgress,
    #[error("job was not found")]
    JobNotFound,
    #[error("no job was run yet")]
    NoJobRun,
    #[error("cannot persist state: {0}")]
    Serde(String),
    #[error("cannot write state: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error(transparent)]
    Poison(#[from] WrappedPoisonError),
}

#[derive(Clone)]
pub struct ServerStopper(pub mpsc::Sender<()>);

impl ServerStopper {
    pub fn stop(&self) -> Result<(), mpsc::TrySendError<()>> {
        self.0.clone().try_send(())
    }
}
