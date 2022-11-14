use crate::Result;
use assert_fs::TempDir;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use valgrind::Protocol;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub token: Option<String>,
    pub working_directory: PathBuf,
    pub protocol: Protocol,
    pub local: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 7070,
            token: None,
            working_directory: TempDir::new().unwrap().into_persistent().to_path_buf(),
            protocol: Default::default(),
            local: true,
        }
    }
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Config> {
    let config = config.as_ref();
    if !config.exists() {
        return Err(crate::error::Error::CannotFindConfig(config.to_path_buf()));
    }

    let contents = std::fs::read_to_string(config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}
