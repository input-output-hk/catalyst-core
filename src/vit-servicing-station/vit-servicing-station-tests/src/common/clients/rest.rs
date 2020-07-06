use thiserror::Error;
use vit_servicing_station_lib::{
    db::models::{funds::Fund, proposals::Proposal},
    v0::api_token::API_TOKEN_HEADER,
};

pub struct RestClient {
    address: String,
    api_token: Option<String>,
}

impl RestClient {
    pub fn new(address: String) -> Self {
        Self {
            address,
            api_token: None,
        }
    }

    pub fn funds(&self) -> Result<Vec<Fund>, RestError> {
        let content = self.get("funds")?.text()?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    pub fn proposals(&self) -> Result<Vec<Proposal>, RestError> {
        let content = self.get("proposals")?.text()?;
        if content.is_empty() {
            return Ok(vec![]);
        }
        serde_json::from_str(&content).map_err(RestError::CannotDeserialize)
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
        let request = self.path(path);
        println!("Request: {}", request);
        let client = reqwest::blocking::Client::new();
        let mut res = client.get(&request);

        if let Some(api_token) = &self.api_token {
            res = res.header(API_TOKEN_HEADER, api_token.to_string());
        }
        let response = res.send()?;
        println!("Response: {:?}", response);
        Ok(response)
    }

    fn path(&self, path: &str) -> String {
        format!("http://{}/api/v0/{}", self.address, path)
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
        let result = response.text();
        println!("{:?}", result);
        Ok(serde_json::from_str(&result?)?)
    }
}

#[derive(Debug, Error)]
pub enum RestError {
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    RequestError(#[from] reqwest::Error),
}
