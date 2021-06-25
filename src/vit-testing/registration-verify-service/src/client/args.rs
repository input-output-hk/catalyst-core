use crate::client::rest::RegistrationRestClient;
use crate::context::State;
use crate::request::Request;
use structopt::StructOpt;
use thiserror::Error;

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
            Self::Health => {
                match rest.is_up() {
                    true => println!("service is up"),
                    false => println!("service is down"),
                }
                Ok(())
            }
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
                println!("{:?}", status_command.exec(rest)?);
                Ok(())
            }
        }
    }
}

#[derive(StructOpt, Debug)]
pub struct StatusCommand {
    /// job id
    #[structopt(short, long)]
    job_id: String,
}

impl StatusCommand {
    pub fn exec(
        self,
        rest: RegistrationVerifyRestClient,
    ) -> Result<Result<State, crate::context::Error>, Error> {
        rest.job_status(self.job_id).map_err(Into::into)
    }
}

#[derive(StructOpt, Debug)]
pub struct NewJobCommand {
    #[structopt(long = "qr")]
    qr: PathBuf,

    #[structopt(long = "pin")]
    pin: String,

    #[structopt(long = "expected-funds")]
    funds: Option<u64>,

    #[structopt(long = "slot-no")]
    slot-no: u64,
}

impl NewJobCommand {
    pub fn exec(self, rest: RegistrationVerifyRestClient) -> Result<String, Error> {
        let request = Request {
            payment_skey: self.payment_skey.clone(),
            payment_vkey: self.payment_vkey.clone(),
            stake_skey: self.stake_skey.clone(),
            stake_vkey: self.stake_vkey,
        };
        rest.job_new(request).map_err(Into::into)
    }
}

#[derive(StructOpt, Debug)]
pub enum FilesCommand {
    List,
}

impl FilesCommand {
    pub fn exec(self, rest: RegistrationRestClient) -> Result<(), Error> {
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
}
