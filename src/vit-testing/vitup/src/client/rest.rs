use crate::config::Config;
use crate::mode::service::manager::{file_lister::FolderDump, State};
use reqwest::blocking::Response;
use thiserror::Error;

pub struct VitupRest {
    token: Option<String>,
    address: String,
}

impl VitupRest {
    pub fn is_up(&self) -> bool {
        let response = reqwest::blocking::get(&self.path("api/health"));

        if response.is_err() {
            return false;
        }

        response.unwrap().status() == 200
    }
}

impl VitupRest {
    pub fn derive_with_port<S: Into<String>>(original: VitupRest, port: S) -> Self {
        Self {
            token: original.token().clone(),
            address: format!("{}:{}", original.address, port.into()),
        }
    }

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

    pub fn path<S: Into<String>>(&self, path: S) -> String {
        format!("{}/{}", self.address, path.into())
    }

    pub fn post_skip_response<S: Into<String>>(&self, local_path: S) -> Result<(), Error> {
        self.post(local_path).map(|_| ())
    }

    pub fn post<S: Into<String>>(&self, local_path: S) -> Result<Response, Error> {
        let path = self.path(local_path);
        println!("Calling: {}", path);
        let client = reqwest::blocking::Client::new();
        client.post(&path).send().map_err(Into::into)
    }

    pub fn get<S: Into<String>>(&self, local_path: S) -> Result<String, Error> {
        let path = self.path(local_path);
        println!("Calling: {}", path);
        Ok(reqwest::blocking::get(&path)?.text()?)
    }
}

pub struct VitupDisruptionRestClient {
    inner: VitupRest,
}

impl From<VitupRest> for VitupDisruptionRestClient {
    fn from(inner: VitupRest) -> Self {
        VitupDisruptionRestClient { inner }
    }
}

impl VitupDisruptionRestClient {
    pub fn get_logs(&self) -> Result<Vec<String>, Error> {
        serde_json::from_str(&self.inner.get("api/control/logs/get")?).map_err(Into::into)
    }

    pub fn clear_logs(&self) -> Result<(), Error> {
        self.inner.post_skip_response("api/control/logs/clear")
    }

    pub fn list_files(&self) -> Result<FolderDump, Error> {
        serde_json::from_str(&self.inner.get("api/control/files/list")?).map_err(Into::into)
    }

    pub fn reset(&self) -> Result<(), Error> {
        self.inner.post_skip_response("api/control/command/reset")
    }

    pub fn reset_with_config(&self, config: &Config) -> Result<Response, Error> {
        let client = reqwest::blocking::Client::new();
        client
            .post(&self.inner.path("api/control/command/reset"))
            .json(&config)
            .send()
            .map_err(Into::into)
    }

    pub fn make_unavailable(&self) -> Result<(), Error> {
        self.inner.post_skip_response("api/control/available/false")
    }

    pub fn set_error_code(&self, error_code: u16) -> Result<(), Error> {
        self.inner
            .post_skip_response(format!("api/control/availabl/error-code/{}", error_code))
    }

    pub fn make_available(&self) -> Result<(), Error> {
        self.inner.post_skip_response("api/control/available/true")
    }

    pub fn set_fund_id(&self, fund_id: u32) -> Result<(), Error> {
        self.inner
            .post_skip_response(format!("api/control/command/fund/id/{}", fund_id))
    }

    pub fn reject_all_fragments(&self) -> Result<(), Error> {
        self.inner
            .post_skip_response("api/control/command/fragments/reject")
    }

    pub fn hold_all_fragments(&self) -> Result<(), Error> {
        self.inner
            .post_skip_response("api/control/command/fragments/pending")
    }

    pub fn accept_all_fragments(&self) -> Result<(), Error> {
        self.inner
            .post_skip_response("api/control/command/fragments/accept")
    }

    pub fn reset_fragments_behavior(&self) -> Result<(), Error> {
        self.inner
            .post_skip_response("api/control/command/fragments/reset")
    }

    pub fn is_up(&self) -> bool {
        if let Ok(path) = self.inner.get("api/health") {
            if let Ok(response) = reqwest::blocking::get(&path) {
                return response.status() == reqwest::StatusCode::OK;
            }
        }
        false
    }
}

pub struct VitupAdminRestClient {
    inner: VitupRest,
}

impl From<VitupRest> for VitupAdminRestClient {
    fn from(inner: VitupRest) -> Self {
        let port = "3030";
        Self {
            inner: VitupRest::derive_with_port(inner, port),
        }
    }
}

impl VitupAdminRestClient {
    pub fn list_files(&self) -> Result<FolderDump, Error> {
        serde_json::from_str(&self.inner.get("api/files/list")?).map_err(Into::into)
    }

    pub fn start_custom(&self, params: Config) -> Result<String, Error> {
        let path = self.inner.path("control/command/start/custom");
        let client = reqwest::blocking::Client::new();
        let response = client.post(&path).json(&params).send()?;
        Ok(response.text()?)
    }

    pub fn start_default(&self) -> Result<String, Error> {
        self.inner
            .post("control/command/start/default")?
            .text()
            .map_err(Into::into)
    }

    pub fn stop(&self) -> Result<String, Error> {
        self.inner
            .post("control/command/stop")?
            .text()
            .map_err(Into::into)
    }

    pub fn status(&self) -> Result<State, Error> {
        let text = self.inner.get("status")?;
        serde_json::from_str(&text).map_err(Into::into)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
