use crate::context::State;
use jortestkit::{prelude::Wait, process::WaitError};
use reqwest::blocking::multipart;
use scheduler_service_lib::SchedulerRestClient;
use thiserror::Error;

pub struct RegistrationVerifyRestClient(SchedulerRestClient);

impl RegistrationVerifyRestClient {
    pub fn new_with_token(token: String, address: String) -> Self {
        Self(SchedulerRestClient::new_with_api_token(token, address))
    }

    pub fn new(address: String) -> Self {
        Self(SchedulerRestClient::new_no_tokens(address))
    }

    pub fn token(&self) -> &Option<String> {
        self.0.api_token()
    }

    pub fn address(&self) -> &String {
        self.0.address()
    }

    pub fn job_new(&self, form: multipart::Form) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.0.path("api/job/new");
        println!("Calling: {}", path);
        let request_builder = self.0.set_header(client.post(&path));
        request_builder
            .multipart(form)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.replace("'\"'", ""))
    }

    pub fn job_status<S: Into<String>>(
        &self,
        id: S,
    ) -> Result<State,Error> {
        self.0.job_status(id).map_err(Into::into)
    }

    pub fn wait_for_job_finish<S: Into<String>>(
        &self,
        id: S,
        mut wait: Wait,
    ) -> Result<State, Error> {
        let job_id = id.into();
        loop {
            let response = self.job_status(job_id.clone())?;
                if let State::Finished { .. } = response {
                    return Ok(response);
                }

            wait.check_timeout()?;
            wait.advance();
        }
    }

    pub fn is_up(&self) -> bool {
        self.0.is_up()
    }
}

#[allow(clippy::from_over_into)]
impl Into<SchedulerRestClient> for RegistrationVerifyRestClient {
    fn into(self) -> SchedulerRestClient {
        self.0
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("qr code not found for job id ({0})")]
    CannotFindQrCode(String),
    #[error("internal rest error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("json response serialization error")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("yaml response serialization error")]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("timeout error")]
    WaitError(#[from] WaitError),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error(transparent)]
    Scheduler(#[from] scheduler_service_lib::RestError)


}
