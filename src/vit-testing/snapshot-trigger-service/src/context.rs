pub type ContextLock = Arc<Mutex<Context>>;
use crate::config::Configuration;
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
    working_dir: PathBuf,
    address: SocketAddr,
    state: State,
}

impl Context {
    pub fn new<P: AsRef<Path>>(config: Configuration, working_dir: P) -> Self {
        Self {
            server_stopper: None,
            address: ([0, 0, 0, 0], config.port).into(),
            config,
            working_dir: working_dir.as_ref().to_path_buf(),
            state: State::Idle,
        }
    }

    pub fn set_server_stopper(&mut self, server_stopper: ServerStopper) {
        self.server_stopper = Some(server_stopper)
    }

    pub fn server_stopper(&self) -> &Option<ServerStopper> {
        &self.server_stopper
    }

    pub fn new_run(&mut self) -> Result<Uuid, Error> {
        match self.state {
            State::Idle | State::Finished { .. } => {
                let id = Uuid::new_v4();
                self.state = State::RequestToStart { job_id: id };
                Ok(id)
            }
            _ => Err(Error::SnaphotInProgress),
        }
    }

    pub fn run_started(&mut self) -> Result<(), Error> {
        match self.state {
            State::RequestToStart { job_id } => {
                self.state = State::Running {
                    job_id,
                    start: Utc::now().naive_utc(),
                };
                Ok(())
            }
            _ => Err(Error::NoRequestToStart),
        }
    }

    pub fn run_finished(&mut self) -> Result<(), Error> {
        match self.state {
            State::Running { job_id, start } => {
                self.state = State::Finished {
                    job_id,
                    start,
                    end: Utc::now().naive_utc(),
                };
                Ok(())
            }
            _ => Err(Error::SnaphotNotStarted),
        }
    }

    pub fn status_by_id(&self, id: Uuid) -> Result<State, Error> {
        match self.state {
            State::Idle => Err(Error::NoJobRun),
            State::RequestToStart { .. } => Ok(self.state),
            State::Running { job_id, .. } => {
                if job_id == id {
                    Ok(self.state)
                } else {
                    Err(Error::JobNotFound)
                }
            }
            State::Finished { job_id, .. } => {
                if job_id == id {
                    Ok(self.state)
                } else {
                    Err(Error::JobNotFound)
                }
            }
        }
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn working_directory(&self) -> &PathBuf {
        &self.working_dir
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

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum State {
    Idle,
    RequestToStart {
        job_id: Uuid,
    },
    Running {
        job_id: Uuid,
        start: NaiveDateTime,
    },
    Finished {
        job_id: Uuid,
        start: NaiveDateTime,
        end: NaiveDateTime,
    },
}

use thiserror::Error;

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

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
/*
mod date_format {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}
*/
