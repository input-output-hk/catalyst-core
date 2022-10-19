use crate::config::JobParameters;
use crate::context::State;
use jortestkit::{prelude::Wait, process::WaitError};
use scheduler_service_lib::FolderDump;
use std::io::Write;
use std::path::Path;
use thiserror::Error;

pub struct SnapshotRestClient {
    token: Option<String>,
    address: String,
}

impl SnapshotRestClient {
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
        format!("{}/{}", self.address, path.into().replace('\"', ""))
    }

    fn get<S: Into<String>>(&self, local_path: S) -> Result<String, Error> {
        let path = self.path(local_path);
        println!("Calling: {}", path);
        let client = reqwest::blocking::Client::new();
        let request = self.set_header(client.get(&path));
        let response = request.send()?;
        if !response.status().is_success() {
            let text = response.text()?;
            return Err(Error::UnexpectedSnapshotRestResponse(text));
        }
        response.text().map_err(Into::into)
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

    pub fn download_snapshot<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        tag: S,
        output: P,
    ) -> Result<(), Error> {
        self.download(
            format!("{}/{}_snapshot.json", id.into().remove_quotas(), tag.into()),
            output,
        )
    }

    pub fn get_snapshot<S: Into<String>>(&self, id: S, tag: S) -> Result<String, Error> {
        self.get(format!(
            "api/job/files/get/{}/{}_snapshot.json",
            id.into().remove_quotas(),
            tag.into()
        ))
    }

    pub fn download_job_status<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        output: P,
    ) -> Result<(), Error> {
        self.download(format!("{}/status.yaml", id.into()), output)
    }

    pub fn get_status<S: Into<String>>(&self, id: S) -> Result<State, Error> {
        let status_string = self.get(format!(
            "api/job/files/get/{}/status.yaml",
            id.into().remove_quotas()
        ))?;
        Ok(serde_yaml::from_str(&status_string)?)
    }

    pub fn download<S: Into<String>, P: AsRef<Path>>(
        &self,
        sub_location: S,
        output: P,
    ) -> Result<(), Error> {
        let content = self.get(format!(
            "api/job/files/get/{}",
            sub_location.into().replace("'\"'", "")
        ))?;
        let mut file = std::fs::File::create(&output)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    pub fn job_new(&self, params: JobParameters) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.path("api/job/new");
        println!("Calling: {}", path);
        let request = self.set_header(client.post(&path));
        request
            .json(&params)
            .send()?
            .text()
            .map_err(Into::into)
            .map(|text| text.remove_quotas())
    }

    pub fn job_status<S: Into<String>>(
        &self,
        id: S,
    ) -> Result<Result<State, crate::context::Error>, Error> {
        let content = self.get(format!("api/job/status/{}", id.into().remove_quotas()))?;
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
        if let Ok(response) = reqwest::blocking::get(&self.path("api/health")) {
            return response.status() == reqwest::StatusCode::OK;
        }
        false
    }
}

pub trait StringExtension {
    fn remove_quotas(self) -> Self;
}

impl StringExtension for String {
    fn remove_quotas(self) -> Self {
        #[allow(clippy::single_char_pattern)]
        self.replace("\"", "")
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
    #[error("error recieved from call: {0}")]
    UnexpectedSnapshotRestResponse(String),
}
