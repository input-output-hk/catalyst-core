use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::ops::Deref;
use std::str::FromStr;
use std::{fmt, fs};

/// Settings environment variables names
const DATABASE_URL: &str = "DATABASE_URL";
const TLS_CERT_FILE: &str = "TLS_CERT_FILE";
const TLS_PRIVATE_KEY_FILE: &str = "TLS_PK_FILE";
const CORS_ALLOWED_ORIGINS: &str = "CORS_ALLOWED_ORIGINS";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, StructOpt)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct ServiceSettings {
    /// Load settings from file
    #[serde(skip)]
    #[structopt(long)]
    pub in_settings_file: Option<String>,

    /// Dump current settings to file
    #[serde(skip)]
    #[structopt(long)]
    pub out_settings_file: Option<String>,

    /// Server binding address
    #[structopt(long, default_value = "0.0.0.0:3030")]
    pub address: SocketAddr,

    #[structopt(flatten)]
    pub tls: Tls,

    #[structopt(flatten)]
    pub cors: Cors,

    /// Database url
    #[structopt(long, env = DATABASE_URL, default_value = "./db/database.sqlite3")]
    pub db_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, StructOpt)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct Tls {
    /// Path to server X.509 certificate chain file, must be PEM-encoded and contain at least 1 item
    #[structopt(long, env = TLS_CERT_FILE)]
    pub cert_file: Option<String>,

    /// Path to server private key file, must be PKCS8 with single PEM-encoded, unencrypted key
    #[structopt(long, env = TLS_PRIVATE_KEY_FILE)]
    pub priv_key_file: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct CorsOrigin(String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AllowedOrigins(Vec<CorsOrigin>);

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, StructOpt)]
#[serde(deny_unknown_fields)]
#[structopt(rename_all = "kebab-case")]
pub struct Cors {
    /// If none provided, echos request origin
    #[serde(default)]
    #[structopt(long, env = CORS_ALLOWED_ORIGINS, parse(try_from_str = parse_allowed_origins))]
    pub allowed_origins: Option<AllowedOrigins>,
    /// If none provided, CORS responses won't be cached
    #[structopt(long)]
    pub max_age_secs: Option<u64>,
}

fn parse_allowed_origins(arg: &str) -> Result<AllowedOrigins, std::io::Error> {
    let mut res: Vec<CorsOrigin> = Vec::new();
    for origin_str in arg.split(';') {
        res.push(CorsOrigin::from_str(origin_str)?);
    }
    Ok(AllowedOrigins(res))
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
                    format!("Invalid schema {}", uri.scheme_str().unwrap()),
                ));
            }
        }
        if let Some(p) = uri.path_and_query() {
            if p.as_str() != "/" {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid schema {}", uri.scheme_str().unwrap()),
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

impl Default for AllowedOrigins {
    fn default() -> Self {
        AllowedOrigins(Vec::new())
    }
}

pub fn load_settings_from_file(file_path: &str) -> Result<ServiceSettings, serde_json::Error> {
    let f = fs::File::open(file_path)
        .unwrap_or_else(|e| panic!("Error reading file {}: {}", file_path, e));
    serde_json::from_reader(&f)
}

pub fn dump_settings_to_file(
    file_path: &str,
    settings: &ServiceSettings,
) -> Result<(), serde_json::Error> {
    let f = fs::File::create(file_path)
        .unwrap_or_else(|e| panic!("Error opening file {}: {}", file_path, e));
    serde_json::to_writer(&f, settings)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::net::SocketAddr;
    use std::str::FromStr;
    use structopt::StructOpt;
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
            "db_url": ""
        }
        "#;

        let config: ServiceSettings = serde_json::from_str(raw_config).unwrap();
        assert_eq!(
            config.address,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );
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
        let settings: ServiceSettings = ServiceSettings::from_iter(&[
            "test",
            "--address",
            "127.0.0.1:3030",
            "--cert-file",
            "foo.bar",
            "--priv-key-file",
            "bar.foo",
            "--db-url",
            "database.sqlite3",
            "--max-age-secs",
            "60",
            "--allowed-origins",
            "https://foo.test;https://test.foo:5050",
        ]);

        assert_eq!(
            settings.address,
            SocketAddr::from_str("127.0.0.1:3030").unwrap()
        );

        assert!(settings.tls.is_loaded());
        assert_eq!(settings.tls.cert_file.unwrap(), "foo.bar");
        assert_eq!(settings.tls.priv_key_file.unwrap(), "bar.foo");
        assert_eq!(settings.db_url, "database.sqlite3");
        assert_eq!(settings.cors.max_age_secs.unwrap(), 60);
        let allowed_origins = settings.cors.allowed_origins.unwrap();
        assert_eq!(allowed_origins.len(), 2);
        assert_eq!(
            allowed_origins[0],
            CorsOrigin("https://foo.test".to_string())
        );
    }

    #[test]
    fn load_settings_from_env() {
        use std::env::set_var;
        set_var(DATABASE_URL, "database.sqlite3");
        set_var(TLS_CERT_FILE, "foo.bar");
        set_var(TLS_PRIVATE_KEY_FILE, "bar.foo");
        set_var(
            CORS_ALLOWED_ORIGINS,
            "https://foo.test;https://test.foo:5050",
        );

        let settings: ServiceSettings = ServiceSettings::from_iter(&[
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
        assert_eq!(settings.db_url, "database.sqlite3");
        assert_eq!(settings.cors.max_age_secs.unwrap(), 60);
        let allowed_origins = settings.cors.allowed_origins.unwrap();
        assert_eq!(allowed_origins.len(), 2);
        assert_eq!(
            allowed_origins[0],
            CorsOrigin("https://foo.test".to_string())
        );
    }
}
