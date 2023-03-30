use crate::logger::{LogLevel, LOG_LEVEL_DEFAULT};
use clap::Args;
use std::net::SocketAddr;

const ADDRESS_DEFAULT: &str = "0.0.0.0:3030";

#[derive(Args, Clone)]
pub struct Settings {
    /// Server binding address
    #[clap(long, default_value = ADDRESS_DEFAULT)]
    pub address: SocketAddr,

    /// Url to the postgres event db
    #[clap(long)]
    pub database_url: String,

    /// Logging level
    #[clap(long, default_value = LOG_LEVEL_DEFAULT)]
    pub log_level: LogLevel,
}
