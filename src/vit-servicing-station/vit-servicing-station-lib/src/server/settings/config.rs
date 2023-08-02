use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use simplelog::LevelFilter;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fmt, fs};

/// Settings environment variables names
const DATABASE_URL: &str = "DATABASE_URL";
const TLS_CERT_FILE: &str = "TLS_CERT_FILE";
const TLS_PRIVATE_KEY_FILE: &str = "TLS_PK_FILE";
const CORS_ALLOWED_ORIGINS: &str = "CORS_ALLOWED_ORIGINS";
const VIT_SERVICE_VERSION_ENV_VARIABLE: &str = "SERVICE_VERSION";

pub(crate) const ADDRESS_DEFAULT: &str = "0.0.0.0:3030";
pub(crate) const DB_URL_DEFAULT: &str = "postgres://localhost";
pub(crate) const BLOCK0_PATH_DEFAULT: &str = "./resources/v0/block0.bin";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Parser)]
#[serde(deny_unknown_fields)]
#[clap(rename_all = "kebab-case")]
pub struct ServiceSettings {
    /// Load settings from file
    #[serde(skip)]
    #[clap(long)]
    pub in_settings_file: Option<String>,

    /// Dump current settings to file
    #[serde(skip)]
    #[clap(long)]
    pub out_settings_file: Option<String>,

    /// Server binding address
    #[clap(long, default_value = ADDRESS_DEFAULT)]
    pub address: SocketAddr,

    #[serde(default)]
    #[clap(flatten)]
    pub tls: Tls,

    #[serde(default)]
    #[clap(flatten)]
    pub cors: Cors,

    /// Database url
    #[clap(long, env = DATABASE_URL, default_value = DB_URL_DEFAULT)]
    pub db_url: String,

    /// block0 static file path
    #[clap(long)]
    pub block0_path: Option<String>,

    /// archive block0 static file path
    #[clap(long)]
    pub block0_paths: Option<PathBuf>,

    /// Enable API Tokens feature
    #[serde(default)]
    #[clap(long)]
    pub enable_api_tokens: bool,

    #[serde(default)]
    #[clap(flatten)]
    pub log: Log,

    #[clap(long, env = VIT_SERVICE_VERSION_ENV_VARIABLE)]
    pub service_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Parser, Default)]
#[serde(deny_unknown_fields)]
#[clap(rename_all = "kebab-case")]
pub struct Tls {
    /// Path to server X.509 certificate chain file, must be PEM-encoded and contain at least 1 item
    #[clap(long, env = TLS_CERT_FILE)]
    pub cert_file: Option<String>,

    /// Path to server private key file, must be PKCS8 with single PEM-encoded, unencrypted key
    #[clap(long, env = TLS_PRIVATE_KEY_FILE)]
    pub priv_key_file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct CorsOrigin(String);

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedOrigins(Vec<CorsOrigin>);

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, Parser)]
#[serde(deny_unknown_fields)]
#[clap(rename_all = "kebab-case")]
pub struct Cors {
    /// If none provided, echos request origin
    #[serde(default)]
    #[clap(long, env = CORS_ALLOWED_ORIGINS, value_parser = parse_allowed_origins)]
    pub allowed_origins: Option<AllowedOrigins>,
    /// If none provided, CORS responses won't be cached
    #[clap(long)]
    pub max_age_secs: Option<u64>,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogLevel {
    Disabled,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Parser)]
#[serde(deny_unknown_fields)]
#[clap(rename_all = "kebab-case")]
pub struct Log {
    /// Output log file path
    #[clap(long)]
    pub log_output_path: Option<PathBuf>,

    /// Application logging level
    #[clap(long)]
    pub log_level: Option<LogLevel>,

    /// Enable the OTLP trace data exporter and set the collector's GRPC endpoint
    #[clap(long = "log-trace-collector-endpoint")]
    pub trace_collector_endpoint: Option<url::Url>,
}

fn parse_allowed_origins(arg: &str) -> Result<AllowedOrigins, std::io::Error> {
    let mut res: Vec<CorsOrigin> = Vec::new();
    for origin_str in arg.split(';') {
        res.push(CorsOrigin::from_str(origin_str)?);
    }
    Ok(AllowedOrigins(res))
}

impl ServiceSettings {
    pub fn override_from(&self, other_settings: &ServiceSettings) -> Self {
        let mut return_settings = self.clone();

        if let Some(in_file) = &other_settings.in_settings_file {
            return_settings.in_settings_file = Some(in_file.clone());
        }

        if let Some(out_file) = &other_settings.out_settings_file {
            return_settings.out_settings_file = Some(out_file.clone());
        }

        if other_settings.address != SocketAddr::from_str(ADDRESS_DEFAULT).unwrap() {
            return_settings.address = other_settings.address;
        }

        if other_settings.tls.is_loaded() {
            return_settings.tls = other_settings.tls.clone();
        }

        if other_settings.cors.allowed_origins.is_some() {
            return_settings.cors.allowed_origins = other_settings.cors.allowed_origins.clone();
        }

        if other_settings.cors.max_age_secs.is_some() {
            return_settings.cors.max_age_secs = other_settings.cors.max_age_secs
        }

        if other_settings.db_url != DB_URL_DEFAULT {
            return_settings.db_url = other_settings.db_url.clone();
        }

        if other_settings.block0_path.is_some() {
            return_settings.block0_path = other_settings.block0_path.clone();
        }

        if other_settings.block0_paths.is_some() {
            return_settings.block0_paths = other_settings.block0_paths.clone();
        }

        if other_settings.log.log_level.is_some() {
            return_settings.log.log_level = other_settings.log.log_level;
        }

        if other_settings.log.log_output_path.is_some() {
            return_settings.log.log_output_path = other_settings.log.log_output_path.clone();
        }

        if !other_settings.service_version.is_empty() {
            return_settings.service_version = other_settings.service_version.clone();
        }

        return_settings.enable_api_tokens = other_settings.enable_api_tokens;

        return_settings
    }
}

impl Tls {
    pub fn is_loaded(&self) -> bool {
        self.priv_key_file.is_some() && self.cert_file.is_some()
    }
}

impl<'de> Deserialize<'de> for CorsOrigin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CorsOriginVisitor;
        impl<'de> Visitor<'de> for CorsOriginVisitor {
            type Value = CorsOrigin;

            fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "an origin in format http[s]://example.com[:3000]",)
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                CorsOrigin::from_str(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(CorsOriginVisitor)
    }
}

impl FromStr for CorsOrigin {
    type Err = std::io::Error;

    fn from_str(origin: &str) -> Result<Self, Self::Err> {
        let uri = warp::http::uri::Uri::from_str(origin).map_err(|invalid_uri| {
            std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("Invalid uri: {}.\n{}", origin, invalid_uri),
            )
        })?;
        if let Some(s) = uri.scheme_str() {
            if s != "http" && s != "https" {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Cors origin invalid schema {}, only [http] and [https] are supported: ",
                        uri.scheme_str().unwrap()
                    ),
                ));
            }
        } else {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Cors origin missing schema, only [http] or [https] are supported",
            ));
        }

        if let Some(p) = uri.path_and_query() {
            if p.as_str() != "/" {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid value {} in cors schema.", p.as_str()),
                ));
            }
        }
        Ok(CorsOrigin(origin.trim_end_matches('/').to_owned()))
    }
}

impl AsRef<str> for CorsOrigin {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Deref for AllowedOrigins {
    type Target = Vec<CorsOrigin>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Disabled => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

impl From<LogLevel> for tracing_subscriber::filter::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Disabled => tracing_subscriber::filter::LevelFilter::OFF,
            LogLevel::Error => tracing_subscriber::filter::LevelFilter::ERROR,
            LogLevel::Warn => tracing_subscriber::filter::LevelFilter::WARN,
            LogLevel::Info => tracing_subscriber::filter::LevelFilter::INFO,
            LogLevel::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
            LogLevel::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
        }
    }
}

impl FromStr for LogLevel {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "disabled" => Ok(Self::Disabled),
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("{} is not a valid log level", s),
            )),
        }
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Disabled => write!(f, "disabled"),
            Self::Error => write!(f, "error"),
            Self::Warn => write!(f, "warn"),
            Self::Info => write!(f, "info"),
            Self::Debug => write!(f, "debug"),
            Self::Trace => write!(f, "trace"),
        }
    }
}

pub fn load_settings_from_file(file_path: &str) -> Result<ServiceSettings, impl std::error::Error> {
    let f = fs::File::open(file_path)?;
    serde_json::from_reader(&f)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))
}

pub fn dump_settings_to_file(
    file_path: &str,
    settings: &ServiceSettings,
) -> Result<(), impl std::error::Error> {
    let f = fs::File::create(file_path)?;
    serde_json::to_writer_pretty(&f, settings)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use tempfile;

    #[test]
    fn cors_origin_from_str() {
        let s = "https://foo.test";
        CorsOrigin::from_str(s).unwrap();
    }

    #[test]
    fn parse_allowed_origins_from_str() {
        let s = "https://foo.test;https://test.foo:5050";
        let res = parse_allowed_origins(s).unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], CorsOrigin::from_str("https://foo.test").unwrap());
        assert_eq!(
            res[1],
            CorsOrigin::from_str("https://test.foo:5050").unwrap()
        );
    }

    #[test]
    fn load_json_configuration() {
        let raw_config = r#"
        {
            "address" : "127.0.0.1:3030",
            "tls" : {
                "cert_file" : "./foo/bar.pem",
                "priv_key_file" : "./bar/foo.pem"
            },
            "cors" : {
                "allowed_origins" : ["https://foo.test"],
                "max_age_secs" : 60
            },
            "db_url": "",
            "block0_path": "./test/bin.test",
            "enable_api_tokens" : true,
            "log" : {
                "log_output_path" : "./server.log",
                "log_level" : "error"
            },
            "service_version" : "v0.2.0"
        }
        "#;

        let config: ServiceSettings = serde_json::from_str(raw_config).unwrap();
        assert_eq!(
            config.address,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );
        assert_eq!(config.block0_path, Some("./test/bin.test".to_string()));
        assert!(config.enable_api_tokens);
        assert_eq!(
            config.log.log_output_path.unwrap(),
            std::path::PathBuf::from_str("./server.log").unwrap()
        );
        assert_eq!(config.log.log_level, Some(LogLevel::Error));
        assert_eq!(config.service_version, "v0.2.0");
        let tls_config = config.tls;
        let cors_config = config.cors;
        assert_eq!(tls_config.cert_file.unwrap(), "./foo/bar.pem");
        assert_eq!(tls_config.priv_key_file.unwrap(), "./bar/foo.pem");
        assert_eq!(
            cors_config.allowed_origins.unwrap()[0],
            CorsOrigin("https://foo.test".to_string())
        );
        assert_eq!(cors_config.max_age_secs.unwrap(), 60);
    }

    #[test]
    fn dump_and_load_settings_to_file() {
        let temp_file_path = tempfile::NamedTempFile::new().unwrap().into_temp_path();
        let settings = ServiceSettings::default();
        let file_path = temp_file_path.to_str().unwrap();
        dump_settings_to_file(file_path, &settings).unwrap();
        let loaded_settings = load_settings_from_file(file_path).unwrap();
        assert_eq!(settings, loaded_settings);
    }

    #[test]
    fn load_settings_from_cli() {
        let settings: ServiceSettings = ServiceSettings::parse_from([
            "test",
            "--address",
            "127.0.0.1:3030",
            "--cert-file",
            "foo.bar",
            "--priv-key-file",
            "bar.foo",
            "--db-url",
            "postgres://localhost",
            "--max-age-secs",
            "60",
            "--allowed-origins",
            "https://foo.test;https://test.foo:5050",
            "--log-output-path",
            "./log.log",
            "--log-level",
            "error",
            "--enable-api-tokens",
            "--service-version",
            "v0.2.0",
        ]);

        assert_eq!(
            settings.address,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );

        assert!(settings.tls.is_loaded());
        assert!(settings.enable_api_tokens);
        assert_eq!(settings.tls.cert_file.unwrap(), "foo.bar");
        assert_eq!(settings.tls.priv_key_file.unwrap(), "bar.foo");
        assert_eq!(settings.db_url, "postgres://localhost");
        assert_eq!(settings.cors.max_age_secs.unwrap(), 60);
        assert_eq!(settings.service_version, "v0.2.0");
        let allowed_origins = settings.cors.allowed_origins.unwrap();
        assert_eq!(allowed_origins.len(), 2);
        assert_eq!(
            allowed_origins[0],
            CorsOrigin("https://foo.test".to_string())
        );
        assert_eq!(
            settings.log.log_output_path.unwrap(),
            std::path::PathBuf::from_str("./log.log").unwrap()
        );
        assert_eq!(settings.log.log_level, Some(LogLevel::Error));
    }

    #[test]
    fn load_settings_from_env() {
        use std::env::set_var;
        set_var(DATABASE_URL, "postgres://localhost");
        set_var(TLS_CERT_FILE, "foo.bar");
        set_var(TLS_PRIVATE_KEY_FILE, "bar.foo");
        set_var(
            CORS_ALLOWED_ORIGINS,
            "https://foo.test;https://test.foo:5050",
        );
        set_var(VIT_SERVICE_VERSION_ENV_VARIABLE, "v0.2.0");

        let settings: ServiceSettings = ServiceSettings::parse_from([
            "test",
            "--address",
            "127.0.0.1:3030",
            "--max-age-secs",
            "60",
        ]);

        assert_eq!(
            settings.address,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );

        assert!(settings.tls.is_loaded());
        assert_eq!(settings.tls.cert_file.unwrap(), "foo.bar");
        assert_eq!(settings.tls.priv_key_file.unwrap(), "bar.foo");
        assert_eq!(settings.db_url, "postgres://localhost");
        assert_eq!(settings.cors.max_age_secs.unwrap(), 60);
        assert_eq!(settings.service_version, "v0.2.0");
        let allowed_origins = settings.cors.allowed_origins.unwrap();
        assert_eq!(allowed_origins.len(), 2);
        assert_eq!(
            allowed_origins[0],
            CorsOrigin("https://foo.test".to_string())
        );
    }

    #[test]
    fn merge_settings() {
        let default = ServiceSettings::default();
        let other_settings = ServiceSettings::parse_from([
            "test",
            "--address",
            "127.0.0.1:8080",
            "--cert-file",
            "foo.bar",
            "--priv-key-file",
            "bar.foo",
            "--db-url",
            "postgres://localhost",
            "--max-age-secs",
            "60",
            "--allowed-origins",
            "https://foo.test;https://test.foo:5050",
            "--block0-path",
            "block0.bin",
            "--enable-api-tokens",
            "--service-version",
            "v0.2.0",
        ]);

        let merged_settings = default.override_from(&other_settings);
        assert_eq!(merged_settings, other_settings);
    }
}
