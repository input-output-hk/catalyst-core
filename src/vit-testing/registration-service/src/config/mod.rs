mod builder;
mod network;

pub use builder::ConfigurationBuilder;
pub use network::NetworkType;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Configuration {
    pub port: u16,
    #[serde(rename = "result-dir")]
    pub result_dir: PathBuf,
    pub jcli: PathBuf,
    #[serde(rename = "cardano-cli")]
    pub cardano_cli: PathBuf,
    #[serde(rename = "voter-registration")]
    pub voter_registration: PathBuf,
    #[serde(rename = "catalyst-toolbox")]
    pub catalyst_toolbox: PathBuf,
    pub network: NetworkType,
    pub token: Option<String>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            port: 7070,
            result_dir: Path::new(".").to_path_buf(),
            network: NetworkType::Mainnet,
            cardano_cli: Path::new("cardano_cli").to_path_buf(),
            voter_registration: Path::new("voter_registration").to_path_buf(),
            jcli: Path::new("jcli").to_path_buf(),
            catalyst_toolbox: Path::new("catalyst-toolbox").to_path_buf(),
            token: None,
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
