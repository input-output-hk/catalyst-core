use super::settings::ServiceSettings;
use dotenv;
use std::collections::HashMap;
use std::env;
use std::env::VarError;
use std::error::Error;

const DATABASE_URL: &str = "DATABASE_URL";
const TLS_CERT_FILE: &str = "TLS_CERT_FILE";
const TLS_PK_FILE: &str = "TLS_PK_FILE";
const CORS_ALLOWED_ORIGINS: &str = "CORS_ALLOWED_ORIGINS";

fn load_settings_values_from_env() -> HashMap<&str, String> {
    let mut map = HashMap::new();
    for key in [
        DATABASE_URL,
        TLS_CERT_FILE,
        TLS_PK_FILE,
        CORS_ALLOWED_ORIGINS,
    ] {
        match env::var(key) {
            Ok(val) => map.insert(key, val),
            _ => (),
        };
    }
    map
}

fn load_settings_from_env() -> ServiceSettings {
    let mut default_config = ServiceSettings::default();
    let env_values = load_settings_from_env();
}
