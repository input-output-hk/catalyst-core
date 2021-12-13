use hyper::StatusCode;
use reqwest::blocking::Response;
use std::collections::HashMap;
use thiserror::Error;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::{
    db::models::{funds::Fund, proposals::Proposal},
    v0::api_token::API_TOKEN_HEADER,
    v0::endpoints::{proposals::ProposalVoteplanIdAndIndexes, service_version::ServiceVersion},
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
        println!("Request: {:#?}", request);
    }

    pub fn log_response(&self, response: &reqwest::blocking::Response) {
        if !self.is_enabled() {
            return;
        }
        println!("Response: {:#?}", response);
    }

    pub fn log_text(&self, content: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Text: {:#?}", content);
    }

    pub fn log_post_body(&self, content: &str) {
        if !self.is_enabled() {
            return;
        }
        println!("Post Body: {}", content);
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled
    }
}

const ORIGIN: &str = "Origin";

#[derive(Debug, Clone)]
pub struct RestClient {
    path_builder: RestPathBuilder,
    api_token: Option<String>,
    logger: RestClientLogger,
    origin: Option<String>,
}

impl RestClient {
    pub fn new(address: String) -> Self {
        Self {
            api_token: None,
            path_builder: RestPathBuilder::new(address),
            logger: RestClientLogger { enabled: true },
            origin: None,
        }
    }

    pub fn health(&self) -> Result<(), RestError> {
        self.get_and_verify_status_code(&self.path_builder.health())
            .map(|_| ())
            .map_err(|_| RestError::ServerIsNotUp)
    }

    pub fn health_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder.health())
            .map_err(RestError::RequestError)
    }

    pub fn funds(&self) -> Result<Fund, RestError> {
        let content = self
            .get_and_verify_status_code(&self.path_builder.funds())?
            .text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(|e| RestError::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn funds_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder.funds())
            .map_err(RestError::RequestError)
    }

    pub fn path_builder(&self) -> &RestPathBuilder {
        &self.path_builder
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, RestError> {
        let content = self
            .get_and_verify_status_code(&self.path_builder.proposals())?
            .text()?;
        self.logger.log_text(&content);
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(|e| RestError::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn proposals_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder.proposals())
            .map_err(RestError::RequestError)
    }

    pub fn proposal(&self, id: &str) -> Result<FullProposalInfo, RestError> {
        let response = self.proposal_raw(id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn proposal_raw(&self, id: &str) -> Result<Response, RestError> {
        self.get(&self.path_builder().proposal(id))
            .map_err(RestError::RequestError)
    }

    pub fn fund(&self, id: &str) -> Result<Fund, RestError> {
        let response = self.fund_raw(id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn proposals_by_voteplan_id_and_index(
        &self,
        request: &[ProposalVoteplanIdAndIndexes],
    ) -> Result<Vec<FullProposalInfo>, RestError> {
        let request_as_string = serde_json::to_string(&request)?;
        serde_json::from_str(&self.post(&self.path_builder().proposals(), request_as_string)?)
            .map_err(RestError::CannotDeserialize)
    }

    pub fn fund_raw(&self, id: &str) -> Result<Response, RestError> {
        self.get(&self.path_builder().fund(id))
            .map_err(RestError::RequestError)
    }

    pub fn challenges_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder().challenges())
            .map_err(RestError::RequestError)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>, RestError> {
        let response = self.challenges_raw()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn genesis(&self) -> Result<Vec<u8>, RestError> {
        Ok(self.genesis_raw()?.bytes()?.to_vec())
    }

    pub fn genesis_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder.genesis())
            .map_err(RestError::RequestError)
    }

    pub fn service_version(&self) -> Result<ServiceVersion, RestError> {
        let response = self.service_version_raw()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn service_version_raw(&self) -> Result<Response, RestError> {
        self.get(&self.path_builder.service_version())
            .map_err(RestError::RequestError)
    }

    pub fn advisor_reviews(
        &self,
        proposal_id: &str,
    ) -> Result<HashMap<String, Vec<AdvisorReview>>, RestError> {
        let response = self.advisor_reviews_raw(proposal_id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.logger.log_text(&content);
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn advisor_reviews_raw(&self, proposal_id: &str) -> Result<Response, RestError> {
        self.get(&self.path_builder.advisor_reviews(proposal_id))
            .map_err(RestError::RequestError)
    }

    pub fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.logger.log_request(path);
        let client = reqwest::blocking::Client::new();
        let mut res = client.get(path);

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        if let Some(origin) = &self.origin {
            res = res.header(ORIGIN, origin.to_string());
        }
        let response = res.send()?;
        self.logger.log_response(&response);
        Ok(response)
    }

    fn get_and_verify_status_code(
        &self,
        path: &str,
    ) -> Result<reqwest::blocking::Response, RestError> {
        let response = self.get(path)?;
        self.verify_status_code(&response)?;
        Ok(response)
    }

    fn verify_status_code(&self, response: &Response) -> Result<(), RestError> {
        if !response.status().is_success() {
            return Err(RestError::ErrorStatusCode(response.status()));
        }
        Ok(())
    }

    pub fn disable_log(&mut self) {
        self.logger.set_enabled(false);
    }

    pub fn enable_log(&mut self) {
        self.logger.set_enabled(true);
    }

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    pub fn set_origin<S: Into<String>>(&mut self, origin: S) {
        self.origin = Some(origin.into());
    }

    fn post(&self, path: &str, data: String) -> Result<String, RestError> {
        let client = reqwest::blocking::Client::new();
        self.logger.log_post_body(&data);

        let mut res = client.post(path).body(String::into_bytes(data));

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        let response = res.send()?;
        self.logger.log_response(&response);
        let content = response.text()?;
        self.logger.log_text(&content);
        Ok(content)
    }
}

#[derive(Debug, Clone)]
pub struct RestPathBuilder {
    address: String,
    root: String,
}

impl RestPathBuilder {
    pub fn new<S: Into<String>>(address: S) -> Self {
        RestPathBuilder {
            root: "/api/v0/".to_string(),
            address: address.into(),
        }
    }

    pub fn proposals(&self) -> String {
        self.path("proposals")
    }

    pub fn funds(&self) -> String {
        self.path("fund")
    }

    pub fn challenges(&self) -> String {
        self.path("challenges")
    }

    pub fn proposal(&self, id: &str) -> String {
        self.path(&format!("proposals/{}", id))
    }

    pub fn fund(&self, id: &str) -> String {
        self.path(&format!("fund/{}", id))
    }

    pub fn advisor_reviews(&self, id: &str) -> String {
        self.path(&format!("reviews/{}", id))
    }

    pub fn genesis(&self) -> String {
        self.path("block0")
    }

    pub fn health(&self) -> String {
        self.path("health")
    }

    pub fn service_version(&self) -> String {
        format!("http://{}{}{}", self.address, "/api/", "vit-version")
    }

    pub fn path(&self, path: &str) -> String {
        format!("http://{}{}{}", self.address, self.root, path)
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
