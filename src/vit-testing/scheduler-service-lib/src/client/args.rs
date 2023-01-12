use super::SchedulerRestClient;
use clap::Parser;
use serde::de::DeserializeOwned;
use thiserror::Error;

pub struct HealthCommand;

impl HealthCommand {
    pub fn exec(rest: SchedulerRestClient) -> Result<(), Error> {
        match rest.is_up() {
            true => println!("env is up"),
            false => println!("env is down"),
        }
        Ok(())
    }
}

#[derive(Parser, Debug)]
pub struct StatusCommand {
    /// job id
    #[clap(short, long)]
    job_id: String,
}

impl StatusCommand {
    pub fn exec<State: DeserializeOwned>(
        self,
        rest: SchedulerRestClient,
    ) -> Result<Result<State, crate::Error>, Error> {
        rest.job_status(self.job_id).map_err(Into::into)
    }
}

#[derive(Parser, Debug)]
pub enum FilesCommand {
    List,
}

impl FilesCommand {
    pub fn exec(self, rest: SchedulerRestClient) -> Result<(), Error> {
        match self {
            Self::List => {
                println!("{}", serde_json::to_string_pretty(&rest.list_files()?)?);
                Ok(())
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("internal rest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("response serialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("rest error")]
    RestError(#[from] super::rest::Error),
    #[error(transparent)]
    Scheduler(#[from] crate::Error),
}
