use crate::context::State;
use jortestkit::{prelude::Wait, process::WaitError};
use reqwest::blocking::multipart;
use thiserror::Error;

pub struct RegistrationVerifyRestClient {
    token: Option<String>,
    address: String,
}

impl RegistrationVerifyRestClient {
    pub fn new_with_token(token: String, address: String) -> Self {
        Self {
            token: Some(token),
            address,
        }
    }

    pub fn new(address: String) -> Self {
        Self {
            token: None,
            address,
        }
    }

    pub fn token(&self) -> &Option<String> {
        &self.token
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    fn path<S: Into<String>>(&self, path: S) -> String {
        format!("{}/{}", self.address, path.into())
    }

    fn get<S: Into<String>>(&self, local_path: S) -> Result<String, Error> {
        let path = self.path(local_path);
        println!("Calling: {}", path);
        let client = reqwest::blocking::Client::new();
        let request = self.set_header(client.get(&path));
        request.send()?.text().map_err(Into::into)
    }

    fn set_header(
        &self,
        request_builder: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        if let Some(token) = &self.token {
            return request_builder.header("API-Token", token);
        }
        request_builder
    }

    pub fn job_new(&self, form: multipart::Form) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.path("api/job/new");
        println!("Calling: {}", path);
        let request_builder = self.set_header(client.post(&path));
        request_builder
            .multipart(form)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.replace("\"", ""))
    }

    pub fn job_status<S: Into<String>>(
        &self,
        id: S,
    ) -> Result<Result<State, crate::context::Error>, Error> {
        let content = self.get(format!("api/job/status/{}", id.into()))?;
        serde_yaml::from_str(&content).map_err(Into::into)
    }

    pub fn wait_for_job_finish<S: Into<String>>(
        &self,
        id: S,
        mut wait: Wait,
    ) -> Result<State, Error> {
        let job_id = id.into();
        loop {
            let response = self.job_status(job_id.clone())?;
            if let Ok(response) = response {
                if let State::Finished { .. } = response {
                    return Ok(response);
                }
            }
            wait.check_timeout()?;
            wait.advance();
        }
    }

    pub fn is_up(&self) -> bool {
        if let Ok(path) = self.get("api/health") {
            if let Ok(response) = reqwest::blocking::get(&path) {
                return response.status() == reqwest::StatusCode::OK;
            }
        }
        false
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
}
