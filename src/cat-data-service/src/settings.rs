use crate::logger::{LogFormat, LogLevel, LOG_FORMAT_DEFAULT, LOG_LEVEL_DEFAULT};
use clap::Args;
use std::net::SocketAddr;

const ADDRESS_DEFAULT: &str = "0.0.0.0:3030";

#[derive(Args, Clone)]
pub struct Settings {
    /// Server binding address
    #[clap(long, default_value = ADDRESS_DEFAULT)]
    pub address: SocketAddr,

    /// Server binding address
    #[clap(long, default_value = Option::None)]
    pub metrics_address: Option<SocketAddr>,

    /// Url to the postgres event db
    #[clap(long, env)]
    pub database_url: String,

    /// Logging format
    #[clap(long, default_value = LOG_FORMAT_DEFAULT)]
    pub log_format: LogFormat,

    /// Logging level
    #[clap(long, default_value = LOG_LEVEL_DEFAULT)]
    pub log_level: LogLevel,
}
