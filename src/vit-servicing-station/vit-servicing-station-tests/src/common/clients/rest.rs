use reqwest::blocking::Response;
use hyper::StatusCode;
use thiserror::Error;
use vit_servicing_station_lib::{
    db::models::{funds::Fund, proposals::Proposal},
    v0::api_token::API_TOKEN_HEADER,
};

#[derive(Debug, Clone)]
pub struct RestClientLogger {
    enabled: bool,
}

impl RestClientLogger {
    pub fn log_request(&self, request: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Request: {}", request);
    }

    pub fn log_response(&self, response: &reqwest::blocking::Response) {
        if !self.is_enabled() {
            return;
        }
        println!("Response: {:?}", response);
    }

    pub fn log_text(&self, content: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Text: {}", content);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}

#[derive(Debug, Clone)]
pub struct RestClient {
    path_builder: RestPathBuilder,
    address: String,
    api_token: Option<String>,
    logger: RestClientLogger,
}

impl RestClient {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_token: None,
            path_builder: Default::default(),
            logger: RestClientLogger { enabled: true },
        }
    }

    pub fn genesis(&self) -> Result<Vec<u8>, RestError> {
        Ok(self.get(&self.path_builder.genesis())?.bytes()?.to_vec())
    }

    pub fn health(&self) -> Result<(), RestError> {
        if self.get("health")?.status().is_success() {
            return Ok(());
        }
        Err(RestError::ServerIsNotUp)
    }

    pub fn funds(&self) -> Result<Vec<Fund>, RestError> {
        let content = self.get_and_verify_status_code("funds")?.text()?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(|e| RestError::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn path_builder(&self) -> &RestPathBuilder {
        &self.path_builder
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, RestError> {
        let content = self.get_and_verify_status_code("proposals")?.text()?;
        self.logger.log_text(&content);
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(|e| RestError::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn proposal(&self, id: &str) -> Result<Proposal, RestError> {
        let content = self.proposal_raw(id)?.text()?;
        println!("Body: {}", content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn proposal_raw(&self, id: &str) -> Result<Response, RestError> {
        self.get(&self.path_builder().proposal(id))
            .map_err(RestError::RequestError)
    }

    pub fn fund(&self, id: &str) -> Result<Fund, RestError> {
        let content = self.fund_raw(id)?.text()?;
        println!("Body: {}", content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn fund_raw(&self, id: &str) -> Result<Response, RestError> {
        self.get(&self.path_builder().fund(id))
            .map_err(RestError::RequestError)
    }

    pub fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let request = self.path(path);
        let client = reqwest::blocking::Client::new();
        let mut res = client.get(&request);

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        let response = res.send()?;
        Ok(response)
    }

    fn get_and_verify_status_code(
        &self,
        path: &str,
    ) -> Result<reqwest::blocking::Response, RestError> {
        let response = self.get(path)?;
        if !response.status().is_success() {
            return Err(RestError::ErrorStatusCode(response.status()));
        }
        Ok(response)
    }

    pub fn disable_log(&mut self) {
        self.logger.set_enabled(false);
    }

    fn path(&self, path: &str) -> String {
        let path = format!("http://{}{}", self.address, path);
        self.logger.log_request(&path);
        path
    }

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    pub fn post(&self, path: &str, data: String) -> Result<serde_json::Value, RestError> {
        let client = reqwest::blocking::Client::new();
        let mut res = client.post(&self.path(path)).body(String::into_bytes(data));

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        let response = res.send()?;
        self.logger.log_response(&response);
        let result = response.text();
        Ok(serde_json::from_str(&result?)?)
    }
}

pub struct RestPathBuilder {
    root: String,
}

impl Default for RestPathBuilder {
    fn default() -> Self {
        RestPathBuilder {
            root: "/api/v0/".to_string(),
        }
    }
}

impl RestPathBuilder {
    pub fn proposals(&self) -> String {
        self.path("proposals")
    }

    pub fn funds(&self) -> String {
        self.path("fund")
    }

    pub fn proposal(&self, id: &str) -> String {
        self.path(&format!("proposals/{}", id))
    }

    pub fn fund(&self, id: &str) -> String {
        self.path(&format!("fund/{}", id))
    }

    pub fn genesis(&self) -> String {
        self.path("block0")
    }

    pub fn graphql(&self) -> String {
        self.path("graphql")
    }

    fn path(&self, path: &str) -> String {
        format!("{}{}", self.root, path)
    }
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("could not deserialize response {text}, due to: {source}")]
    CannotDeserializeResponse {
        source: serde_json::Error,
        text: String,
    },
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    RequestError(#[from] reqwest::Error),
    #[error("server is not up")]
    ServerIsNotUp,
    #[error("Error code recieved: {0}")]
    ErrorStatusCode(StatusCode),
}
