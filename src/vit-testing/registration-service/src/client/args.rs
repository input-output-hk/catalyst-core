use crate::client::rest::RegistrationRestClient;
use crate::context::State;
use crate::request::Request;
use scheduler_service_lib::{FilesCommand, HealthCommand, StatusCommand};
use structopt::StructOpt;
use thiserror::Error;

/// registration-cli
#[derive(StructOpt, Debug)]
pub struct RegistrationServiceCliCommand {
    /// access token
    #[structopt(short, long, env = "REGISTRATION_TOKEN")]
    token: Option<String>,

    /// registration service endpoint
    #[structopt(short, long, env = "REGISTRATION_ENDPOINT")]
    endpoint: String,

    #[structopt(subcommand)]
    command: Command,
}

impl RegistrationServiceCliCommand {
    pub fn exec(self) -> Result<(), Error> {
        let rest = match self.token {
            Some(token) => RegistrationRestClient::new_with_token(token, self.endpoint),
            None => RegistrationRestClient::new(self.endpoint),
        };

        self.command.exec(rest)
    }
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// check if registration service is up
    Health,
    /// download jobs artifacts
    Files(FilesCommand),
    /// jobs related operations
    Job(JobCommand),
}

impl Command {
    pub fn exec(self, rest: RegistrationRestClient) -> Result<(), Error> {
        match self {
            Self::Health => HealthCommand::exec(rest.into()).map_err(Into::into),
            Self::Files(files_command) => files_command.exec(rest.into()).map_err(Into::into),
            Self::Job(job_command) => job_command.exec(rest),
        }
    }
}

#[derive(StructOpt, Debug)]
pub enum JobCommand {
    /// start new job
    New(NewJobCommand),
    /// get job status
    Status(StatusCommand),
}

impl JobCommand {
    pub fn exec(self, rest: RegistrationRestClient) -> Result<(), Error> {
        match self {
            Self::New(new_job_command) => {
                println!("{}", new_job_command.exec(rest)?);
                Ok(())
            }
            Self::Status(status_command) => {
                println!("{:?}", status_command.exec::<State>(rest.into())?);
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct NewJobCommand {
    /// payment.skey file
    #[structopt(long = "payment-skey")]
    payment_skey: String,
    /// payment.vkey file
    #[structopt(long = "payment-vkey")]
    payment_vkey: String,
    /// stake.skey file
    #[structopt(long = "stake-skey")]
    stake_skey: String,
    /// stake.vkey file
    #[structopt(long = "stake-vkey")]
    stake_vkey: String,
    /// vote.skey file
    #[structopt(long = "vote-vkey")]
    vote_skey: Option<String>,

    /// delegation
    #[structopt(long = "delegation-1")]
    delegation_1: Option<String>,

    /// delegation
    #[structopt(long = "delegation-2")]
    delegation_2: Option<String>,

    /// delegation
    #[structopt(long = "delegation-3")]
    delegation_3: Option<String>,
}

impl NewJobCommand {
    pub fn exec(self, rest: RegistrationRestClient) -> Result<String, Error> {
        let request = Request {
            payment_skey: self.payment_skey,
            payment_vkey: self.payment_vkey,
            stake_skey: self.stake_skey,
            stake_vkey: self.stake_vkey,
            legacy_skey: self.vote_skey,
            delegation_1: self.delegation_1,
            delegation_2: self.delegation_2,
            delegation_3: self.delegation_3,
        };
        rest.job_new(request).map_err(Into::into)
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
    #[error("rest error")]
    CLi(#[from] scheduler_service_lib::CliError),
}
