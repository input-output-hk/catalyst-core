use crate::DbConfig;
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub db: DbConfig,
    pub token: Option<String>,
}
