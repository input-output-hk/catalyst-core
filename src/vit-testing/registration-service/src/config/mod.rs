mod builder;
mod network;

use std::net::SocketAddr;
pub use builder::ConfigurationBuilder;
pub use network::NetworkType;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    #[serde(flatten)]
    pub inner: scheduler_service_lib::Configuration,
    pub jcli: PathBuf,
    #[serde(rename = "cardano-cli")]
    pub cardano_cli: PathBuf,
    #[serde(rename = "voter-registration")]
    pub voter_registration: PathBuf,
    #[serde(rename = "catalyst-toolbox")]
    pub catalyst_toolbox: PathBuf,
    pub network: NetworkType,
}

impl Configuration {
    pub fn working_directory(&self) -> &Option<PathBuf> {
        &self.inner.working_directory
    }

    pub fn result_directory(&self) -> &PathBuf {
        &self.inner.result_dir
    }

    pub fn address(&self) -> SocketAddr {
        self.inner.address
    }

    pub fn address_mut(&mut self) -> &mut SocketAddr {
        &mut self.inner.address
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            inner: scheduler_service_lib::Configuration {
                result_dir: Path::new(".").to_path_buf(),
                address: ([0, 0, 0, 0], 7070).into(),
                api_token: None,
                admin_token: None,
                working_directory: None,
            },
            network: NetworkType::Mainnet,
            cardano_cli: Path::new("cardano_cli").to_path_buf(),
            voter_registration: Path::new("voter_registration").to_path_buf(),
            jcli: Path::new("jcli").to_path_buf(),
            catalyst_toolbox: Path::new("catalyst-toolbox").to_path_buf(),
        }
    }
}

pub fn read_config<P: AsRef<Path>>(config: P) -> Result<Configuration, Error> {
    let contents = std::fs::read_to_string(&config)?;
    serde_json::from_str(&contents).map_err(Into::into)
}

pub fn write_config<P: AsRef<Path>>(config: Configuration, path: P) -> Result<(), Error> {
    use std::io::Write;
    let mut file = std::fs::File::create(&path)?;
    file.write_all(serde_json::to_string_pretty(&config)?.as_bytes())
        .map_err(Into::into)
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
