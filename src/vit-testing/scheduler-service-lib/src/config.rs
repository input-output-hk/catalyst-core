use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone, Debug, Eq, PartialEq)]
pub struct Configuration {
    #[serde(rename = "result-dir")]
    pub result_dir: PathBuf,
    pub address: SocketAddr,
    pub api_token: Option<String>,
    pub admin_token: Option<String>,
}
