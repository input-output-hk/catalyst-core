use crate::load::config::NodeLoadConfig;
use jortestkit::load::Configuration;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum RequestType {
    #[serde(alias = "vote")]
    Vote,
    #[serde(alias = "account")]
    Account,
    #[serde(alias = "fund")]
    Fund,
    #[serde(alias = "challenges")]
    Challenges,
    #[serde(alias = "challenge")]
    Challenge,
    #[serde(alias = "proposal")]
    Proposal,
    #[serde(alias = "proposals")]
    Proposals,
    #[serde(alias = "settings")]
    Settings,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub account: Configuration,
    pub vote: NodeLoadConfig,
    pub fund: Configuration,
    pub challenges: Configuration,
    pub proposal: Configuration,
    pub challenge: Configuration,
    pub settings: Configuration,
}
