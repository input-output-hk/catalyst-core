use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    pub token: Option<String>,
    pub ideascale: bool,
    #[serde(alias = "working-dir")]
    pub working_dir: PathBuf,
    #[serde(default)]
    pub protocol: valgrind::Protocol,
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Configuration, Error> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("cannot parse configuration")]
    ParseConfiguration(#[from] serde_json::Error),
    #[error("cannot read configuration: {0:?}")]
    ReadConfiguration(PathBuf),
    #[error("cannot spawn command")]
    SpawnCommand(#[from] std::io::Error),
}
