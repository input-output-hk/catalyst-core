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

#[cfg(test)]
mod test {
    use super::{CorsOrigin, ServiceSettings};
    use std::net::SocketAddr;
    use std::str::FromStr;

    #[test]
    fn load_simple_configuration() {
        let raw_config = r#"
        {
            "listen" : "127.0.0.1:3030",
            "tls" : {
                "cert_file" : "./foo/bar.pem",
                "priv_key_file" : "./bar/foo.pem"
            },
            "cors" : {
                "allowed_origins" : ["https://foo.test"],
                "max_age_secs" : 60
            }
        }
        "#;

        let config: ServiceSettings = serde_json::from_str(raw_config).unwrap();
        assert_eq!(
            config.listen,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );
        let tls_config = config.tls.clone().unwrap();
        let cors_config = config.cors.clone().unwrap();
        assert_eq!(tls_config.cert_file, "./foo/bar.pem");
        assert_eq!(tls_config.priv_key_file, "./bar/foo.pem");
        assert_eq!(
            cors_config.allowed_origins[0],
            CorsOrigin("https://foo.test".to_string())
        );
        assert_eq!(cors_config.max_age_secs.unwrap(), 60);
    }
}
