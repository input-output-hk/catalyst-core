use crate::context::State;
use crate::file_lister::FolderDump;
use crate::request::Request;
use jortestkit::{prelude::Wait, process::WaitError};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

pub struct RegistrationRestClient {
    token: Option<String>,
    address: String,
}

impl RegistrationRestClient {
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

    pub fn list_files(&self) -> Result<FolderDump, Error> {
        serde_json::from_str(&self.get("api/job/files/list")?).map_err(Into::into)
    }

    pub fn download_qr<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        output_dir: P,
    ) -> Result<PathBuf, Error> {
        let folder_dump = self.list_files()?;
        let id = id.into();
        let qr_code_file_sub_url = folder_dump
            .find_qr(id.clone())
            .ok_or_else(|| Error::CannotFindQrCode(id.clone()))?;
        let file_name = Path::new(&qr_code_file_sub_url)
            .file_name()
            .ok_or(Error::CannotFindQrCode(id))?;
        let output_path = output_dir.as_ref().join(file_name);
        self.download(Self::rem_first(qr_code_file_sub_url), output_path.clone())?;
        Ok(output_path)
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

    pub fn get_catalyst_sk<S: Into<String>>(&self, id: S) -> Result<String, Error> {
        self.get(format!(
            "api/job/files/get/{}/catalyst-vote.skey",
            id.into()
        ))
    }

    pub fn job_new(&self, request: Request) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.path("api/job/new");
        println!("Calling: {}", path);
        let request_builder = self.set_header(client.post(&path));
        request_builder
            .json(&request)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.replace("'\"'", ""))
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

    fn rem_first(value: &str) -> &str {
        let mut chars = value.chars();
        chars.next();
        chars.as_str()
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
