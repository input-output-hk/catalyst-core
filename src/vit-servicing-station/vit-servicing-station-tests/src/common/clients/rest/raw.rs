use super::Error;
use super::{RestClientLogger, RestPathBuilder};
use reqwest::blocking::Response;
use std::time::Duration;
use url::Url;
use vit_servicing_station_lib::v0::api_token::API_TOKEN_HEADER;

#[derive(Debug, Clone)]
pub struct RestClient {
    path_builder: RestPathBuilder,
    api_token: Option<String>,
    logger: RestClientLogger,
    origin: Option<String>,
    timeout: Option<Duration>,
}

const ORIGIN: &str = "Origin";

impl RestClient {
    pub fn new(url: Url) -> Self {
        Self {
            api_token: None,
            path_builder: RestPathBuilder::new(url),
            logger: RestClientLogger::default(),
            origin: None,
            timeout: None,
        }
    }

    pub fn health(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.health()).map_err(Into::into)
    }

    pub fn funds(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.funds()).map_err(Into::into)
    }

    pub fn proposals(&self, group: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.proposals_with_group(group))
            .map_err(Into::into)
    }

    pub fn put_snapshot_info(&self, tag: &str, content: String) -> Result<Response, Error> {
        self.put(
            &self.path_builder.clone().admin().snapshot_info(tag),
            content,
        )
        .map_err(Into::into)
    }

    pub fn put_raw_snapshot(&self, tag: &str, content: String) -> Result<Response, Error> {
        self.put(
            &self.path_builder.clone().admin().raw_snapshot(tag),
            content,
        )
        .map_err(Into::into)
    }

    pub fn snapshot_tags(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.snapshot_tags())
            .map_err(Into::into)
    }

    pub fn voter_info(&self, tag: &str, key: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.snapshot_voter_info(tag, key))
            .map_err(Into::into)
    }

    pub fn delegator_info(&self, tag: &str, key: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.snapshot_delegator_info(tag, key))
            .map_err(Into::into)
    }

    pub fn proposal(&self, id: &str, group: &str) -> Result<Response, Error> {
        self.get(&self.path_builder().proposal(id, group))
            .map_err(Into::into)
    }

    pub fn proposals_by_voteplan_id_and_index(
        &self,
        request_as_string: &str,
    ) -> Result<Response, Error> {
        self.post(
            &self.path_builder().proposals(),
            request_as_string.to_string(),
        )
        .map_err(Into::into)
    }

    pub fn fund(&self, id: &str) -> Result<Response, Error> {
        self.get(&self.path_builder().fund(id)).map_err(Into::into)
    }

    pub fn challenges(&self) -> Result<Response, Error> {
        self.get(&self.path_builder().challenges())
            .map_err(Into::into)
    }

    pub fn genesis(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.genesis()).map_err(Into::into)
    }

    pub fn service_version(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.service_version())
            .map_err(Into::into)
    }

    pub fn advisor_reviews(&self, proposal_id: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.advisor_reviews(proposal_id))
            .map_err(Into::into)
    }

    pub fn search(&self, query: impl Into<String>) -> Result<Response, Error> {
        self.post(&self.path_builder.search(), query.into())
            .map_err(Into::into)
    }

    pub fn search_count(&self, query: impl Into<String>) -> Result<Response, Error> {
        self.post(&self.path_builder.search_count(), query.into())
            .map_err(Into::into)
    }

    pub fn client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()}

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    pub fn set_origin<S: Into<String>>(&mut self, origin: S) {
        self.origin = Some(origin.into());
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Some(timeout);
    }

    pub fn disable_log(&mut self) {
        self.logger.set_enabled(false);
    }

    pub fn enable_log(&mut self) {
        self.logger.set_enabled(true);
    }

    pub fn log_response(&self, response: &Response) {
        self.logger.log_response(response);
    }

    pub fn log_text(&self, content: &str) {
        self.logger.log_text(content);
    }

    pub fn path_builder(&self) -> &RestPathBuilder {
        &self.path_builder
    }

    fn post(
        &self,
        path: &str,
        data: String,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.logger.log_post_body(&data);

        let mut res = self.client()?.post(path).body(String::into_bytes(data));

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        let response = res.send()?;
        Ok(response)
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.logger.log_request(path);
        let mut res = self.client()?.get(path);

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

    fn put(&self, path: &str, body: String) -> Result<reqwest::blocking::Response, reqwest::Error> {
        self.logger.log_request(path);
        let mut res = self.client()?.put(path).body(body);

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        if let Some(origin) = &self.origin {
            res = res.header(ORIGIN, origin.to_string());
        }
        if let Some(timeout) = self.timeout {
            res = res.timeout(timeout);
        }
        let response = res.send()?;
        self.logger.log_response(&response);
        Ok(response)
    }
}
