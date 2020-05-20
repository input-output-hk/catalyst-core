use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServiceSettings {
    pub listen: SocketAddr,
    /// Enables TLS and disables plain HTTP if provided
    pub tls: Option<Tls>,
    /// Enables CORS if provided
    pub cors: Option<Cors>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Tls {
    /// Path to server X.509 certificate chain file, must be PEM-encoded and contain at least 1 item
    pub cert_file: String,
    /// Path to server private key file, must be PKCS8 with single PEM-encoded, unencrypted key
    pub priv_key_file: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorsOrigin(String);

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Cors {
    /// If none provided, echos request origin
    #[serde(default)]
    pub allowed_origins: Vec<CorsOrigin>,
    /// If none provided, CORS responses won't be cached
    pub max_age_secs: Option<u64>,
}
