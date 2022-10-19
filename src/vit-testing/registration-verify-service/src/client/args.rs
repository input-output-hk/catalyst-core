use crate::client::rest::RegistrationVerifyRestClient;
use reqwest::blocking::multipart::Form;
use scheduler_service_lib::{HealthCommand, StatusCommand};
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;
use crate::context::State;

#[derive(StructOpt, Debug)]
pub struct RegistrationVerifyServiceCliCommand {
    /// token, which is necessary to perform admin operations
    #[structopt(short, long, env = "REGISTRATION_VERIFY_TOKEN")]
    token: Option<String>,

    /// snapshot endpoint
    #[structopt(short, long, env = "REGISTRATION_VERIFY_ENDPOINT")]
    endpoint: String,

    #[structopt(subcommand)]
    command: Command,
}

impl RegistrationVerifyServiceCliCommand {
    pub fn exec(self) -> Result<(), Error> {
        let rest = match self.token {
            Some(token) => RegistrationVerifyRestClient::new_with_token(token, self.endpoint),
            None => RegistrationVerifyRestClient::new(self.endpoint),
        };

        self.command.exec(rest)
    }
}

#[derive(StructOpt, Debug)]
pub enum Command {
    /// check if registration verify service is up
    Health,
    // job related commands
    Job(JobCommand),
}

impl Command {
    pub fn exec(self, rest: RegistrationVerifyRestClient) -> Result<(), Error> {
        match self {
            Self::Health => HealthCommand::exec(rest.into()).map_err(Into::into),
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
    pub fn exec(self, rest: RegistrationVerifyRestClient) -> Result<(), Error> {
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
    /// path to qr code
    #[structopt(long = "qr")]
    qr: PathBuf,

    /// pin
    #[structopt(long = "pin")]
    pin: String,

    /// expected funds (needed for assertion)
    #[structopt(long = "expected-funds")]
    funds: u64,

    /// snapshot threshold
    #[structopt(long = "threshold")]
    threshold: u64,

    /// snapshot slot number
    #[structopt(long = "slot-no")]
    slot_no: Option<u64>,
}

impl NewJobCommand {
    pub fn exec(self, rest: RegistrationVerifyRestClient) -> Result<String, Error> {
        let mut form = Form::new()
            .text("pin", self.pin)
            .text("funds", self.funds.to_string())
            .text("threshold", self.threshold.to_string())
            .file("qr", &self.qr)?;

        if let Some(slot_no) = self.slot_no {
            form = form.text("slot-no", slot_no.to_string());
        }

        rest.job_new(form).map_err(Into::into)
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
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Cli(#[from] scheduler_service_lib::CliError)
}
