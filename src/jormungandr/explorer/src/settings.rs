use crate::logging::{LogFormat, LogOutput, LogSettings};
use clap::Parser;
use jormungandr_lib::interfaces::{Cors, Tls};
use lazy_static::lazy_static;
use serde::{de, de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::{fs::File, net::SocketAddr, path::PathBuf, str::FromStr};
use thiserror::Error;
use tonic::transport::Uri;
use tracing::metadata::LevelFilter;
use url::Url;

const DEFAULT_QUERY_DEPTH_LIMIT: usize = 15;
const DEFAULT_QUERY_COMPLEXITY_LIMIT: usize = 100;

lazy_static! {
    pub static ref LOG_FILTER_LEVEL_POSSIBLE_VALUES: Vec<&'static str> = {
        [
            tracing::metadata::LevelFilter::OFF,
            tracing::metadata::LevelFilter::TRACE,
            tracing::metadata::LevelFilter::DEBUG,
            tracing::metadata::LevelFilter::INFO,
            tracing::metadata::LevelFilter::WARN,
            tracing::metadata::LevelFilter::ERROR,
        ]
        .iter()
        .map(|name| name.to_string().to_ascii_lowercase())
        .map(|name| &*Box::leak(name.into_boxed_str()))
        .collect()
    };
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Format(#[from] serde_yaml::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Invalid host")]
    InvalidHost,
}

pub struct Settings {
    pub node: Uri,
    pub binding_address: SocketAddr,
    pub address_bech32_prefix: String,
    pub query_depth_limit: usize,
    pub query_complexity_limit: usize,
    pub tls: Option<Tls>,
    pub cors: Option<Cors>,
    pub log_settings: Option<LogSettings>,
}

impl Settings {
    pub fn load() -> Result<Settings, Error> {
        let cmd = CommandLine::parse();
        let file: Config = cmd
            .config
            .as_ref()
            .map(|file_path| -> Result<Config, Error> {
                serde_yaml::from_reader(File::open(file_path)?).map_err(Error::from)
            })
            .transpose()?
            .unwrap_or_default();

        let node = cmd
            .node
            .clone()
            .or_else(|| file.node.clone())
            .unwrap_or_else(|| "127.0.0.1:8299".parse().unwrap());

        let binding_address = cmd
            .binding_address
            .or(file.binding_address)
            .unwrap_or_else(|| "0.0.0.0:3030".parse().unwrap());

        let address_bech32_prefix = cmd
            .address_bech32_prefix
            .clone()
            .or_else(|| file.address_bech32_prefix.clone())
            .unwrap_or_else(|| "addr".to_string());

        let query_depth_limit = cmd
            .query_depth_limit
            .or(file.query_depth_limit)
            .unwrap_or(DEFAULT_QUERY_DEPTH_LIMIT);

        let query_complexity_limit = cmd
            .query_complexity_limit
            .or(file.query_complexity_limit)
            .unwrap_or(DEFAULT_QUERY_COMPLEXITY_LIMIT);

        let log_settings = Some(Self::log_settings(&cmd, &file));

        let tls = file.tls;
        let cors = file.cors;

        Ok(Settings {
            node,
            binding_address,
            address_bech32_prefix,
            query_depth_limit,
            query_complexity_limit,
            tls,
            cors,
            log_settings,
        })
    }

    fn log_settings(cmd: &CommandLine, file: &Config) -> LogSettings {
        // Start with default config
        let mut settings = LogSettings::default();

        //  Read log settings from the config file path.
        if let Some(cfg) = file.logs.as_ref() {
            if let Some(level) = cfg.level {
                settings.level = level;
            }
            if let Some(format) = cfg.format {
                settings.format = format;
            }
            if let Some(output) = &cfg.output {
                settings.output = output.clone();
            }
            if cfg.trace_collector_endpoint.is_some() {
                settings.trace_collector_endpoint = cfg.trace_collector_endpoint.clone();
            }
        }

        // If the command line specifies log arguments, they override everything
        // else.
        if let Some(output) = &cmd.log_output {
            settings.output = output.clone();
        }
        if let Some(level) = cmd.log_level {
            settings.level = level;
        }
        if let Some(format) = cmd.log_format {
            settings.format = format;
        }
        if cmd.log_trace_collector_endpoint.is_some() {
            settings.trace_collector_endpoint = cmd.log_trace_collector_endpoint.clone();
        }

        settings
    }
}

#[derive(Debug, Parser)]
#[clap(name = "config")]
struct CommandLine {
    #[clap(long)]
    pub node: Option<Uri>,
    #[clap(long)]
    pub binding_address: Option<SocketAddr>,
    #[clap(long)]
    pub address_bech32_prefix: Option<String>,
    #[clap(long)]
    pub query_depth_limit: Option<usize>,
    #[clap(long)]
    pub query_complexity_limit: Option<usize>,

    pub config: Option<PathBuf>,
    /// Set log messages minimum severity. If not configured anywhere, defaults to "info".
    #[clap(
        long = "log-level",
        value_parser = log_level_parse,
    )]
    pub log_level: Option<LevelFilter>,

    /// Set format of the log emitted. Can be "json" or "plain".
    /// If not configured anywhere, defaults to "plain".
    #[clap(long = "log-format", value_parser = LogFormat::from_str)]
    pub log_format: Option<LogFormat>,

    /// Set format of the log emitted. Can be "stdout", "stderr" or a file path preceeded by '@' (e.g. @./explorer.log).
    /// If not configured anywhere, defaults to "stderr".
    #[clap(long = "log-output", value_parser = LogOutput::from_str)]
    pub log_output: Option<LogOutput>,

    /// Enable the OTLP trace data exporter and set the collector's GRPC endpoint.
    #[clap(long)]
    pub log_trace_collector_endpoint: Option<Url>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub tls: Option<Tls>,
    pub cors: Option<Cors>,
    #[serde(default, deserialize_with = "deserialize_uri_string")]
    pub node: Option<Uri>,
    pub binding_address: Option<SocketAddr>,
    pub address_bech32_prefix: Option<String>,
    pub query_depth_limit: Option<usize>,
    pub query_complexity_limit: Option<usize>,
    pub logs: Option<ConfigLogSettings>,
}

fn deserialize_uri_string<'de, D>(deserializer: D) -> Result<Option<Uri>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let s: String = de::Deserialize::deserialize(deserializer)?;
    Ok(Some(s.parse().map_err(D::Error::custom)?))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigLogSettings {
    #[serde(with = "filter_level_opt_serde")]
    pub level: Option<LevelFilter>,
    pub format: Option<LogFormat>,
    pub output: Option<LogOutput>,
    pub trace_collector_endpoint: Option<Url>,
}

mod filter_level_opt_serde {
    use super::*;

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<LevelFilter>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|variant| {
                variant.parse().map_err(|_| {
                    D::Error::unknown_variant(&variant, &LOG_FILTER_LEVEL_POSSIBLE_VALUES)
                })
            })
            .transpose()
    }

    pub fn serialize<S: Serializer>(
        data: &Option<LevelFilter>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        data.map(|level| level.to_string()).serialize(serializer)
    }
}

fn log_level_parse(level: &str) -> Result<LevelFilter, String> {
    level
        .parse()
        .map_err(|_| format!("Unknown log level value: '{}'", level))
}
