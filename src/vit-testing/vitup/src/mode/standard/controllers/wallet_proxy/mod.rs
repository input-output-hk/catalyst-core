#![allow(dead_code)]
#![allow(non_local_definitions)]

mod controller;
mod settings;
mod spawn_params;

use chain_impl_mockchain::fragment::FragmentId;

use hersir::builder::NodeAlias;
use jormungandr_automation::jormungandr::grpc::client::MockClientError;
use jormungandr_automation::jormungandr::RestError;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub use controller::{Error as WalletProxyControllerError, WalletProxyController};
pub use spawn_params::WalletProxySpawnParams;

pub use self::settings::WalletProxySettings;

pub type Result<T> = std::result::Result<T, Error>;

#[allow(non_local_definitions)]
#[derive(custom_debug::Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    WalletProxyController(#[from] WalletProxyControllerError),
    #[error(transparent)]
    RestError(#[from] RestError),
    #[error(transparent)]
    SerializationError(#[from] yaml_rust::scanner::ScanError),
    #[error(transparent)]
    GrpcError(#[from] MockClientError),
    #[error("cannot create file {path}")]
    CannotCreateFile {
        path: PathBuf,
        #[source]
        cause: io::Error,
    },
    #[error("cannot write YAML into {path}")]
    CannotWriteYamlFile {
        path: PathBuf,
        #[source]
        cause: serde_yaml::Error,
    },
    #[error("cannot spawn the node")]
    CannotSpawnNode(#[source] io::Error),
    #[error("port already binded: {0}")]
    PortAlreadyBinded(u16),
    #[error("no wallet proxy defined in settings")]
    NoWalletProxiesDefinedInSettings,
    #[error("no explorer defined in settings")]
    NoExplorerDefinedInSettings,
    #[error("fragment logs in an invalid format")]
    InvalidFragmentLogs(#[source] serde_json::Error),
    #[error("node stats in an invalid format")]
    InvalidNodeStats(#[source] RestError),
    #[error("network stats in an invalid format")]
    InvalidNetworkStats(#[source] serde_json::Error),
    #[error("leaders ids in an invalid format")]
    InvalidEnclaveLeaderIds(#[source] serde_json::Error),
    #[error("node '{alias}' failed to start after {} s", .duration.as_secs())]
    NodeFailedToBootstrap {
        alias: String,
        duration: Duration,
        #[debug(skip)]
        logs: Vec<String>,
    },
    #[error("node '{alias}' failed to shutdown, message: {message}")]
    NodeFailedToShutdown {
        alias: String,
        message: String,
        #[debug(skip)]
        logs: Vec<String>,
    },
    #[error("fragment '{fragment_id}' not in the mempool of the node '{alias}'")]
    FragmentNotInMemPoolLogs {
        alias: String,
        fragment_id: FragmentId,
        #[debug(skip)]
        logs: Vec<String>,
    },
    #[error("fragment '{fragment_id}' is pending for too long ({} s) for node '{alias}'", .duration.as_secs())]
    FragmentIsPendingForTooLong {
        fragment_id: FragmentId,
        duration: Duration,
        alias: String,
        #[debug(skip)]
        logs: Vec<String>,
    },
}
