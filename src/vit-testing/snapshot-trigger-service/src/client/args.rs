use crate::client::rest::SnapshotRestClient;
use crate::config::JobParameters;
use crate::ContextState;
use scheduler_service_lib::{CliError, FilesCommand, HealthCommand};
use clap::Parser;
use thiserror::Error;

#[derive(Parser, Debug)]
pub struct TriggerServiceCliCommand {
    /// access token, which is necessary to perform client operations
    #[clap(short, long, env = "SNAPSHOT_TOKEN")]
    token: Option<String>,

    /// snapshot endpoint
    #[clap(short, long, env = "SNAPSHOT_ENDPOINT")]
    endpoint: String,

    #[clap(subcommand)]
    command: Command,
}

impl TriggerServiceCliCommand {
    pub fn exec(self) -> Result<(), Error> {
        let rest = match self.token {
            Some(token) => SnapshotRestClient::new_with_token(token, self.endpoint),
            None => SnapshotRestClient::new(self.endpoint),
        };

        self.command.exec(rest)
    }
}

#[derive(Parser, Debug)]
pub enum Command {
    /// check if snapshot service is up
    Health,
    /// retrieve files from snapshot (snapshot outcome etc.)
    Files(FilesCommand),
    /// job related commands
    Job(JobCommand),
}

impl Command {
    pub fn exec(self, rest: SnapshotRestClient) -> Result<(), Error> {
        match self {
            Self::Health => HealthCommand::exec(rest.into()).map_err(Into::into),
            Self::Files(files_command) => files_command.exec(rest.into()).map_err(Into::into),
            Self::Job(job_command) => job_command.exec(rest),
        }
    }
}

#[derive(Parser, Debug)]
pub enum JobCommand {
    /// start new job
    New(NewJobCommand),
    /// get job status
    Status(StatusCommand),
}

impl JobCommand {
    pub fn exec(self, rest: SnapshotRestClient) -> Result<(), Error> {
        match self {
            Self::New(new_job_command) => {
                println!("{}", new_job_command.exec(rest)?);
                Ok(())
            }
            Self::Status(status_command) => {
                println!("{:?}", status_command.exec(rest)?);
                Ok(())
            }
        }
    }
}

#[derive(Parser, Debug)]
pub struct StatusCommand {
    /// job id
    #[clap(short, long)]
    job_id: String,
}

impl StatusCommand {
    pub fn exec(self, rest: SnapshotRestClient) -> Result<ContextState, Error> {
        rest.get_status(self.job_id).map_err(Into::into)
    }
}

#[derive(Parser, Debug)]
pub struct NewJobCommand {
    /// slot no
    #[clap(short, long)]
    slot_no: Option<u64>,
    /// tag
    #[clap(short, long)]
    tag: Option<String>,
}

impl NewJobCommand {
    pub fn exec(self, rest: SnapshotRestClient) -> Result<String, Error> {
        let params = JobParameters {
            slot_no: self.slot_no,
            tag: self.tag,
        };
        rest.job_new(params).map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("internal rest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("response serialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("rest error")]
    Rest(#[from] super::rest::Error),
    #[error("rest error")]
    Context(#[from] crate::context::Error),
    #[error(transparent)]
    Cli(#[from] CliError),
}
