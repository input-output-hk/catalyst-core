mod logger;
mod path;
mod raw;

use crate::common::clients::rest::path::RestPathBuilder;
use crate::common::snapshot::{Snapshot, VotingPower};
use hyper::StatusCode;
use logger::RestClientLogger;
pub use raw::RestClient as RawRestClient;
use reqwest::blocking::Response;
use std::collections::HashMap;
use thiserror::Error;
use url::Url;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_lib::{
    db::models::{funds::Fund, proposals::Proposal},
    v0::endpoints::{proposals::ProposalVoteplanIdAndIndexes, service_version::ServiceVersion},
};

#[derive(Debug, Clone)]
pub struct RestClient {
    raw: RawRestClient,
}

impl From<&ServiceSettings> for RestClient {
    fn from(settings: &ServiceSettings) -> Self {
        let url = {
            let scheme = {
                if settings.tls.cert_file.is_some() {
                    "https"
                } else {
                    "http"
                }
            };
            //we accepted ServiceSettings struct in constructor, so address should be proper
            //SockerAddr struct, therefore we won't have any problems with parsing result
            format!("{}://{}", scheme, settings.address)
                .parse()
                .unwrap()
        };
        Self::new(url)
    }
}

#[allow(clippy::from_over_into)]
impl Into<RawRestClient> for RestClient {
    fn into(self) -> RawRestClient {
        self.raw
    }
}

impl RestClient {
    pub fn new(url: Url) -> Self {
        Self {
            raw: RawRestClient::new(url),
        }
    }

    pub fn health(&self) -> Result<(), Error> {
        self.verify_status_code(&self.raw.health()?)
            .map_err(|_| Error::ServerIsNotUp)
    }

    pub fn funds(&self) -> Result<Fund, Error> {
        let response = self.raw.funds()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(|e| Error::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn path_builder(&self) -> &RestPathBuilder {
        self.raw.path_builder()
    }

    pub fn put_snapshot(&self, snapshot: &Snapshot) -> Result<(), Error> {
        let content = serde_json::to_string(&snapshot.content)?;
        self.verify_status_code(&self.raw.put_snapshot(&snapshot.tag, content)?)
    }

    pub fn proposal(&self, id: &str) -> Result<Proposal, Error> {
        let response = self.raw.proposal(id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(|e| Error::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, Error> {
        let response = self.raw.proposals()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(|e| Error::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn snapshot_tags(&self) -> Result<Vec<String>, Error> {
        let response = self.raw.snapshot_tags()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn voting_power(&self, tag: &str, key: &str) -> Result<Vec<VotingPower>, Error> {
        let response = self.raw.voting_power(tag, key)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn fund(&self, id: &str) -> Result<Fund, Error> {
        let response = self.raw.fund(id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn proposals_by_voteplan_id_and_index(
        &self,
        request: &[ProposalVoteplanIdAndIndexes],
    ) -> Result<Vec<FullProposalInfo>, Error> {
        let request_as_string = serde_json::to_string(&request)?;
        serde_json::from_str(
            &self
                .raw
                .proposals_by_voteplan_id_and_index(&request_as_string)?
                .text()?,
        )
        .map_err(Error::CannotDeserialize)
    }

    pub fn challenges(&self) -> Result<Vec<Challenge>, Error> {
        let response = self.raw.challenges()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn genesis(&self) -> Result<Vec<u8>, Error> {
        Ok(self.raw.genesis()?.bytes()?.to_vec())
    }

    pub fn service_version(&self) -> Result<ServiceVersion, Error> {
        let response = self.raw.service_version()?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn advisor_reviews(
        &self,
        proposal_id: &str,
    ) -> Result<HashMap<String, Vec<AdvisorReview>>, Error> {
        let response = self.raw.advisor_reviews(proposal_id)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    fn verify_status_code(&self, response: &Response) -> Result<(), Error> {
        if !response.status().is_success() {
            return Err(Error::ErrorStatusCode(response.status()));
        }
        Ok(())
    }

    pub fn disable_log(&mut self) {
        self.raw.disable_log();
    }

    pub fn enable_log(&mut self) {
        self.raw.enable_log();
    }

    pub fn set_api_token(&mut self, token: String) {
        self.raw.set_api_token(token);
    }

    pub fn set_origin<S: Into<String>>(&mut self, origin: S) {
        self.raw.set_origin(origin);
    }
}

#[derive(Debug, Error)]
pub enum Error {
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
