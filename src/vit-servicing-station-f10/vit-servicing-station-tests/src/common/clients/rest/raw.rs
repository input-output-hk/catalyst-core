use reqwest::blocking::Response;

use super::Error;
use super::{RestClientLogger, RestPathBuilder};
use url::Url;
use vit_servicing_station_lib::v0::api_token::API_TOKEN_HEADER;

#[derive(Debug, Clone)]
pub struct RestClient {
    path_builder: RestPathBuilder,
    api_token: Option<String>,
    logger: RestClientLogger,
    origin: Option<String>,
}

const ORIGIN: &str = "Origin";

impl RestClient {
    pub fn new(url: Url) -> Self {
        Self {
            api_token: None,
            path_builder: RestPathBuilder::new(url),
            logger: RestClientLogger::default(),
            origin: None,
        }
    }

    pub fn health(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.health())
            .map_err(Error::RequestError)
    }

    pub fn funds(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.funds())
            .map_err(Error::RequestError)
    }

    pub fn proposals(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.proposals())
            .map_err(Error::RequestError)
    }

    pub fn put_snapshot(&self, tag: &str, content: String) -> Result<Response, Error> {
        self.put(&self.path_builder.clone().admin().snapshot(tag), content)
            .map_err(Error::RequestError)
    }

    pub fn snapshot_tags(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.snapshot_tags())
            .map_err(Error::RequestError)
    }

    pub fn voting_power(&self, tag: &str, key: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.snapshot_voting_power(tag, key))
            .map_err(Error::RequestError)
    }

    pub fn proposal(&self, id: &str) -> Result<Response, Error> {
        self.get(&self.path_builder().proposal(id))
            .map_err(Error::RequestError)
    }

    pub fn proposals_by_voteplan_id_and_index(
        &self,
        request_as_string: &str,
    ) -> Result<Response, Error> {
        self.post(
            &self.path_builder().proposals(),
            request_as_string.to_string(),
        )
        .map_err(Error::RequestError)
    }

    pub fn fund(&self, id: &str) -> Result<Response, Error> {
        self.get(&self.path_builder().fund(id))
            .map_err(Error::RequestError)
    }

    pub fn challenges(&self) -> Result<Response, Error> {
        self.get(&self.path_builder().challenges())
            .map_err(Error::RequestError)
    }

    pub fn genesis(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.genesis())
            .map_err(Error::RequestError)
    }

    pub fn service_version(&self) -> Result<Response, Error> {
        self.get(&self.path_builder.service_version())
            .map_err(Error::RequestError)
    }

    pub fn advisor_reviews(&self, proposal_id: &str) -> Result<Response, Error> {
        self.get(&self.path_builder.advisor_reviews(proposal_id))
            .map_err(Error::RequestError)
    }

    pub fn client(&self) -> Result<reqwest::blocking::Client, reqwest::Error> {
        reqwest::blocking::Client::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .map_err(Into::into)
    }

    pub fn set_api_token(&mut self, token: String) {
        self.api_token = Some(token);
    }

    pub fn set_origin<S: Into<String>>(&mut self, origin: S) {
        self.origin = Some(origin.into());
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
        let response = res.send()?;
        self.logger.log_response(&response);
        Ok(response)
    }
}
