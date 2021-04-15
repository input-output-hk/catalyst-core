use super::config::ServiceSettings;
use super::config::{Cors, Log, Tls};
use std::net::SocketAddr;
use std::str::FromStr;

impl Default for ServiceSettings {
    fn default() -> Self {
        Self {
            in_settings_file: None,
            out_settings_file: None,
            address: SocketAddr::from_str("0.0.0.0:3030").unwrap(),
            tls: Tls::default(),
            cors: Cors::default(),
            db_url: "./db/database.sqlite3".to_string(),
            block0_path: "./resources/v0/block0.bin".to_string(),
            enable_api_tokens: false,
            log: Log::default(),
            service_version: "".to_string(),
        }
    }
}
