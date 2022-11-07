use crate::FolderDump;
use jortestkit::process::WaitError;
use jortestkit::web::api_token::API_TOKEN_HEADER;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub struct SchedulerRestClient {
    api_token: Option<String>,
    admin_token: Option<String>,
    address: String,
}

impl SchedulerRestClient {
    pub fn new_with_api_token(token: String, address: String) -> Self {
        Self::new(Some(token), None, address)
    }

    pub fn new_with_admin_token(token: String, address: String) -> Self {
        Self::new(None, Some(token), address)
    }

    pub fn new_no_tokens(address: String) -> Self {
        Self::new(None, None, address)
    }

    pub fn new(api_token: Option<String>, admin_token: Option<String>, address: String) -> Self {
        Self {
            api_token,
            admin_token,
            address,
        }
    }

    pub fn address(&self) -> &String {
        &self.address
    }

    pub fn path<S: Into<String>>(&self, path: S) -> String {
        format!("{}/{}", self.address, path.into())
    }

    pub fn get<S: Into<String>>(&self, local_path: S) -> Result<String, Error> {
        let path = self.path(local_path);
        println!("Calling: {}", path);
        let client = reqwest::blocking::Client::new();
        let request = self.set_header(client.get(&path));
        request.send()?.text().map_err(Into::into)
    }

    pub fn set_header(
        &self,
        request_builder: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        if let Some(token) = self.api_token() {
            return request_builder.header(API_TOKEN_HEADER, token);
        }
        if let Some(token) = self.admin_token() {
            return request_builder.header(API_TOKEN_HEADER, token);
        }
        request_builder
    }

    pub fn list_files(&self) -> Result<FolderDump, Error> {
        serde_json::from_str(&self.get("api/job/files/list")?).map_err(Into::into)
    }

    pub fn download<S: Into<String>, P: AsRef<Path>>(
        &self,
        sub_location: S,
        output: P,
    ) -> Result<(), Error> {
        let local_path = format!("api/job/files/get/{}", sub_location.into());
        let path = self.path(local_path);
        let client = reqwest::blocking::Client::new();
        let request = self.set_header(client.get(&path));
        let bytes = request.send()?.bytes()?;
        let mut file = std::fs::File::create(&output)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    pub fn job_new<Request: Serialize>(&self, request: Request) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.path("api/job/new");
        println!("Calling: {}", path);
        let request_builder = self.set_header(client.post(&path));
        #[allow(clippy::single_char_pattern)]
        request_builder
            .json(&request)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.replace("\"", ""))
    }

    pub fn job_status<S: Into<String>, State: DeserializeOwned>(
        &self,
        id: S,
    ) -> Result<State, Error> {
        #[allow(clippy::single_char_pattern)]
        let content = self.get(format!("api/job/status/{}", id.into().replace("\"", "")))?;
        serde_yaml::from_str(&content).map_err(Into::into)
    }

    pub fn is_up(&self) -> bool {
        if let Ok(response) = reqwest::blocking::get(&self.path("api/health")) {
            return response.status() == reqwest::StatusCode::OK;
        }
        false
    }
    pub fn api_token(&self) -> &Option<String> {
        &self.api_token
    }
    pub fn admin_token(&self) -> &Option<String> {
        &self.admin_token
    }
}

#[derive(Error, Debug)]
pub enum Error {
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
