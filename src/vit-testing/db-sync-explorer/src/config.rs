use crate::{DbConfig, MockConfig};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub address: SocketAddr,
    pub db: Db,
    pub token: Option<String>,
}

impl Config {
    pub(crate) fn mock_config(&self) -> Option<&MockConfig> {
        if let Db::Mock(config) = &self.db {
            Some(config)
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum Db {
    Db(DbConfig),
    Mock(MockConfig),
}
