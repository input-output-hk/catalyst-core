use crate::context::State;
use crate::request::Request;
use jortestkit::{prelude::Wait, process::WaitError, string::rem_first};
use scheduler_service_lib::{FolderDump, SchedulerRestClient};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

pub struct RegistrationRestClient(SchedulerRestClient);

impl RegistrationRestClient {
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

    pub fn list_files(&self) -> Result<FolderDump, Error> {
        self.0.list_files().map_err(Into::into)
    }

    pub fn download_qr<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        output_dir: P,
    ) -> Result<PathBuf, Error> {
        let folder_dump = self.list_files()?;
        let id = id.into();
        let qr_code_file_sub_url = folder_dump
            .find_file_with_extension(id.clone(), "png".to_string())
            .ok_or_else(|| Error::CannotFindQrCode(id.clone()))?;
        let file_name = Path::new(&qr_code_file_sub_url)
            .file_name()
            .ok_or(Error::CannotFindQrCode(id))?;
        let output_path = output_dir.as_ref().join(file_name);
        self.download(rem_first(qr_code_file_sub_url), output_path.clone())?;
        Ok(output_path)
    }

    pub fn download<S: Into<String>, P: AsRef<Path>>(
        &self,
        sub_location: S,
        output: P,
    ) -> Result<(), Error> {
        self.0.download(sub_location, output).map_err(Into::into)
    }

    pub fn get_catalyst_sk<S: Into<String>>(&self, id: S) -> Result<String, Error> {
        self.0
            .get(format!(
                "api/job/files/get/{}/catalyst-vote.skey",
                id.into()
            ))
            .map_err(Into::into)
    }

    pub fn job_new(&self, request: Request) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.0.path("api/job/new");
        println!("Calling: {}", path);
        let request_builder = self.0.set_header(client.post(&path));
        #[allow(clippy::single_char_pattern)]
        request_builder
            .json(&request)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.replace("\"", ""))
    }

    pub fn job_status<S: Into<String>>(
        &self,
        id: S,
    ) -> Result<Result<State, crate::context::Error>, Error> {
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
        self.0.is_up()
    }
}

#[allow(clippy::from_over_into)]
impl Into<SchedulerRestClient> for RegistrationRestClient {
    fn into(self) -> SchedulerRestClient {
        self.0
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("qr code not found for job id ({0})")]
    CannotFindQrCode(String),
    #[error("internal rest error")]
    Reqwest(#[from] reqwest::Error),
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
    Rest(#[from] scheduler_service_lib::RestError),
}
