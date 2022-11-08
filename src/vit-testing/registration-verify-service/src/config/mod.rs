mod network;

pub use network::NetworkType;

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    #[serde(rename = "snapshot-token")]
    pub snapshot_token: String,
    #[serde(rename = "snapshot-address")]
    pub snapshot_address: String,
    pub jcli: PathBuf,
    #[serde(rename = "initial-snapshot-job-id")]
    pub snapshot_job_id: Option<String>,
    pub network: NetworkType,
    #[serde(flatten)]
    pub inner: scheduler_service_lib::Configuration,
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Configuration, Error> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse configuration")]
    CannotParseConfiguration(#[from] serde_json::Error),
    #[error("cannot read configuration: {0:?}")]
    CannotReadConfiguration(PathBuf),
    #[error("cannot spawn command")]
    CannotSpawnCommand(#[from] std::io::Error),
}
