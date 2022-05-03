use super::config::ServiceSettings;
use super::config::{Cors, Log, Tls, ADDRESS_DEFAULT, BLOCK0_PATH_DEFAULT, DB_URL_DEFAULT};
use std::net::SocketAddr;
use std::str::FromStr;

impl Default for ServiceSettings {
    fn default() -> Self {
        Self {
            in_settings_file: None,
            out_settings_file: None,
            address: SocketAddr::from_str(ADDRESS_DEFAULT).unwrap(),
            tls: Tls::default(),
            cors: Cors::default(),
            db_url: DB_URL_DEFAULT.to_string(),
            block0_path: BLOCK0_PATH_DEFAULT.to_string(),
            enable_api_tokens: false,
            log: Log::default(),
            service_version: "".to_string(),
        }
    }
}
