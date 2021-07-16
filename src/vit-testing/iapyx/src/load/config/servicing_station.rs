use jortestkit::load::Configuration;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    #[serde(alias = "fund")]
    Fund,
    #[serde(alias = "challenges")]
    Challenges,
    #[serde(alias = "proposal")]
    Proposal,
    #[serde(alias = "proposals")]
    Proposals,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub config: HashMap<RequestType, Configuration>,
    pub criterion: Option<u8>,
    pub address: String,
    pub use_https: bool,
}

impl Config {
    pub fn get(&self, request_type: RequestType) -> Result<Configuration, Error> {
        self.config
            .get(&request_type)
            .cloned()
            .ok_or(Error::CannotFindConfigurationFor(request_type))
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("cannot find configuration for request type: {0:?}")]
    CannotFindConfigurationFor(RequestType),
}
