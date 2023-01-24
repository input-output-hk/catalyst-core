mod logger;
mod path;
mod raw;
mod search;

pub use raw::RestClient as RawRestClient;
pub use search::SearchRequestBuilder;
use vit_servicing_station_lib::v0::endpoints::snapshot::DelegatorInfo;

use crate::common::clients::rest::path::RestPathBuilder;
use crate::common::raw_snapshot::RawSnapshot;
use crate::common::snapshot::{Snapshot, VoterInfo};
use hyper::StatusCode;
use logger::RestClientLogger;
use reqwest::blocking::Response;
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use url::Url;
use vit_servicing_station_lib::db::models::challenges::Challenge;
use vit_servicing_station_lib::db::models::community_advisors_reviews::AdvisorReview;
use vit_servicing_station_lib::db::models::proposals::FullProposalInfo;
use vit_servicing_station_lib::server::settings::ServiceSettings;
use vit_servicing_station_lib::v0::endpoints::search::requests::{SearchQuery, SearchResponse};
use vit_servicing_station_lib::{
    db::models::funds::Fund,
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
            //SocketAddr struct, therefore we won't have any problems with parsing result
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

    pub fn put_snapshot_info(&self, snapshot: &Snapshot) -> Result<(), Error> {
        let content = serde_json::to_string(&snapshot.content)?;
        self.verify_status_code(&self.raw.put_snapshot_info(&snapshot.tag, content)?)
    }

    pub fn put_raw_snapshot(&self, raw_snapshot: &RawSnapshot) -> Result<(), Error> {
        let content = serde_json::to_string(&raw_snapshot.content)?;
        self.verify_status_code(&self.raw.put_raw_snapshot(&raw_snapshot.tag, content)?)
    }

    pub fn proposal(&self, id: &str, group: &str) -> Result<FullProposalInfo, Error> {
        let response = self.raw.proposal(id, group)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(|e| Error::CannotDeserializeResponse {
            source: e,
            text: content.clone(),
        })
    }

    pub fn proposals(&self, group: &str) -> Result<Vec<FullProposalInfo>, Error> {
        let response = self.raw.proposals(group)?;
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

    pub fn voter_info(&self, tag: &str, key: &str) -> Result<VoterInfo, Error> {
        let response = self.raw.voter_info(tag, key)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn delegator_info(&self, tag: &str, key: &str) -> Result<DelegatorInfo, Error> {
        let response = self.raw.delegator_info(tag, key)?;
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

    pub fn search(&self, search: SearchQuery) -> Result<SearchResponse, Error> {
        let response = self.raw.search(serde_json::to_string(&search)?)?;
        self.verify_status_code(&response)?;
        let content = response.text()?;
        self.raw.log_text(&content);
        serde_json::from_str(&content).map_err(Error::CannotDeserialize)
    }

    pub fn search_count(&self, search: SearchQuery) -> Result<i64, Error> {
        let response = self.raw.search_count(serde_json::to_string(&search)?)?;
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

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.raw.set_timeout(timeout);
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
