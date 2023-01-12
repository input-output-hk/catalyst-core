use crate::{
    blockcfg::HeaderHash,
    settings::{
        logging::{LogFormat, LogOutput},
        start::config::TrustedPeer,
        LOG_FILTER_LEVEL_POSSIBLE_VALUES,
    },
};
use clap::Parser;
use multiaddr::Multiaddr;
use std::{net::SocketAddr, path::PathBuf, str::FromStr};
use tracing::level_filters::LevelFilter;

fn trusted_peer_from_json(json: &str) -> Result<TrustedPeer, serde_json::Error> {
    serde_json::from_str(json)
}

#[derive(Parser, Debug)]
pub struct StartArguments {
    /// Path to the blockchain pool storage directory
    #[clap(long = "storage", value_parser = PathBuf::from_str)]
    pub storage: Option<PathBuf>,

    /// Set the node config (in YAML format) to use as general configuration
    #[clap(long = "config", value_parser = PathBuf::from_str)]
    pub node_config: Option<PathBuf>,

    /// Set the secret node config (in YAML format).
    #[clap(long = "secret", value_parser = PathBuf::from_str)]
    pub secret: Option<PathBuf>,

    /// Path to the genesis block (the block0) of the blockchain
    #[clap(long = "genesis-block", value_parser = PathBuf::from_str)]
    pub block_0_path: Option<PathBuf>,

    /// set a trusted peer in the multiformat format (e.g.: '/ip4/192.168.0.1/tcp/8029')
    ///
    /// This is the trusted peer the node will connect to initially to download the initial
    /// block0 and fast fetch missing blocks since last start of the node.
    #[clap(long = "trusted-peer", value_parser = trusted_peer_from_json)]
    pub trusted_peer: Vec<TrustedPeer>,

    /// set the genesis block hash (the hash of the block0) so we can retrieve the
    /// genesis block (and the blockchain configuration) from the existing storage
    /// or from the network.
    #[clap(long = "genesis-block-hash", value_parser = HeaderHash::from_str)]
    pub block_0_hash: Option<HeaderHash>,

    /// Enable the Prometheus metrics exporter.
    #[cfg(feature = "prometheus-metrics")]
    #[clap(long = "enable-prometheus")]
    pub prometheus_enabled: bool,

    /// The address to listen from and accept connection from. This is the
    /// public address that will be distributed to other peers of the network.
    #[clap(long = "public-address")]
    pub public_address: Option<Multiaddr>,

    /// Specifies the address the node will listen to to receive p2p connection.
    /// Can be left empty and the node will listen to whatever value was given
    /// to `public_address`.
    #[clap(long = "listen-address")]
    pub listen_address: Option<SocketAddr>,
}

#[derive(Parser, Debug)]
pub struct RestArguments {
    /// REST API listening address.
    /// If not configured anywhere, defaults to REST API being disabled
    #[clap(name = "rest-listen")]
    pub listen: Option<SocketAddr>,
}

#[derive(Parser, Debug)]
pub struct JRpcArguments {
    /// JRPC API listening address.
    /// If not configured anywhere, defaults to JRPC API being disabled
    #[clap(name = "jrpc-listen")]
    pub listen: Option<SocketAddr>,
}

#[derive(Parser, Debug)]
#[clap(name = "jormungandr")]
pub struct CommandLine {
    /// Set log messages minimum severity. If not configured anywhere, defaults to "info".
    #[clap(
        long = "log-level",
        value_parser = log_level_parse,
        value_names = &*LOG_FILTER_LEVEL_POSSIBLE_VALUES
    )]
    pub log_level: Option<LevelFilter>,

    /// Set format of the log emitted. Can be "json" or "plain".
    /// If not configured anywhere, defaults to "plain".
    #[clap(long = "log-format", value_parser = LogFormat::from_str)]
    pub log_format: Option<LogFormat>,

    /// Set where the log will be emitted. Can be "stdout", "stderr",
    /// a file path preceeded by '@' (e.g. @./jormungandr.log)
    /// or "journald" (linux with systemd only, must be enabled during compilation).
    ///
    /// If not configured anywhere, defaults to "stderr"
    #[clap(long = "log-output", value_parser = LogOutput::from_str)]
    pub log_output: Option<LogOutput>,

    /// Enable the OTLP trace data exporter and set the collector's GRPC endpoint.
    #[clap(long = "log-trace-collector-endpoint")]
    pub trace_collector_endpoint: Option<url::Url>,

    /// report all the rewards in the reward distribution history
    ///
    /// NOTE: this will slowdown the epoch transition computation and will add
    /// add a lot of items for in-memory operations, this is not recommended to set
    #[clap(long = "rewards-report-all")]
    pub rewards_report_all: bool,

    #[clap(flatten)]
    pub rest_arguments: RestArguments,

    #[clap(flatten)]
    pub jrpc_arguments: JRpcArguments,

    #[clap(flatten)]
    pub start_arguments: StartArguments,

    /// display full version details (software version, source version, targets and compiler used)
    #[clap(long = "full-version")]
    pub full_version: bool,

    /// display the sources version, allowing to check the source's hash used to compile this executable.
    /// this option is useful for scripting retrieving the logs of the version of this application.
    #[clap(long = "source-version")]
    pub source_version: bool,

    /// Initialize the storage and exit, useful to check that the storage has been set up correctly.
    #[clap(long = "storage-check")]
    pub storage_check: bool,
}

impl CommandLine {
    /// load the command arguments from the command line args
    ///
    /// on error during reading the command line arguments, the
    /// function will print an error message and will terminate
    /// the process.
    ///
    pub fn load() -> Self {
        Self::parse()
    }
}

fn log_level_parse(level: &str) -> Result<LevelFilter, String> {
    level
        .parse()
        .map_err(|_| format!("Unknown log level value: '{}'", level))
}
