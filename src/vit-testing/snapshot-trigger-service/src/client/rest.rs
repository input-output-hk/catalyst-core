use crate::config::JobParameters;
use crate::ContextState;
use jortestkit::string::StringExtension;
use jortestkit::{prelude::Wait, process::WaitError};
use scheduler_service_lib::{FolderDump, SchedulerRestClient};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Debug)]
pub struct SnapshotRestClient(SchedulerRestClient);

impl SnapshotRestClient {
    pub fn new_with_token(token: String, address: String) -> Self {
        Self(SchedulerRestClient::new(Some(token), None, address))
    }

    pub fn new(address: String) -> Self {
        Self(SchedulerRestClient::new(None, None, address))
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

    pub fn download_snapshot<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        tag: S,
        output: P,
    ) -> Result<(), Error> {
        self.0
            .download(
                format!("{}/{}_snapshot.json", id.into().remove_quotas(), tag.into()),
                output,
            )
            .map_err(Into::into)
    }

    pub fn get_snapshot<S: Into<String>>(&self, id: S, tag: S) -> Result<String, Error> {
        self.0
            .get(format!(
                "api/job/files/get/{}/{}_snapshot.json",
                id.into().remove_quotas(),
                tag.into()
            ))
            .map_err(Into::into)
    }

    pub fn download_job_status<S: Into<String>, P: AsRef<Path>>(
        &self,
        id: S,
        output: P,
    ) -> Result<(), Error> {
        self.0
            .download(format!("{}/status.yaml", id.into()), output)
            .map_err(Into::into)
    }

    pub fn get_status<S: Into<String>>(&self, id: S) -> Result<ContextState, Error> {
        self.0.job_status(id).map_err(Into::into)
    }

    #[instrument]
    pub fn job_new(&self, params: JobParameters) -> Result<String, Error> {
        let client = reqwest::blocking::Client::new();
        let path = self.0.path("api/job/new");
        debug!("Calling: {path}");
        let request = self.0.set_header(client.post(&path));
        let response = request.json(&params).send()?;

        if response.status() != 200 {
            return Err(Error::UnexpectedSnapshotRestResponse {
                path,
                text: response.text()?,
            });
        }

        response
            .text()
            .map_err(Into::into)
            .map(|text| text.remove_quotas())
    }

    pub fn wait_for_job_finish<S: Into<String>>(
        &self,
        id: S,
        mut wait: Wait,
    ) -> Result<ContextState, Error> {
        let job_id = id.into();
        loop {
            if let Ok(response) = self.get_status(job_id.clone()) {
                if let ContextState::Finished { .. } = response {
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
impl Into<SchedulerRestClient> for SnapshotRestClient {
    fn into(self) -> SchedulerRestClient {
        self.0
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
    #[error("error received from call on endpoint '{path}': {text}")]
    UnexpectedSnapshotRestResponse { path: String, text: String },
    #[error(transparent)]
    Inner(#[from] scheduler_service_lib::RestError),
}
