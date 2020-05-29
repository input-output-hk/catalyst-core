use super::settings::ServiceSettings;
use crate::server_settings::{Cors, Tls};
use std::net::SocketAddr;
use std::str::FromStr;

impl Default for ServiceSettings {
    fn default() -> Self {
        Self {
            address: SocketAddr::from_str("0.0.0.0:3030").unwrap(),
            tls: Tls {
                cert_file: None,
                priv_key_file: None,
            },
            cors: Cors {
                max_age_secs: None,
                allowed_origins: None,
            },
            db_url: "./db/database.sqlite3".to_string(),
        }
    }
}
