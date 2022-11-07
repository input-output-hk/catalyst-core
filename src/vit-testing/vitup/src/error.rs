use crate::mode::standard::{
    VitStationControllerError, WalletProxyControllerError, WalletProxyError,
};
use chain_impl_mockchain::ledger::Block0Error;
use hersir::controller::NodeError;
use jormungandr_automation::testing::ConsumptionBenchmarkError;
use jormungandr_automation::testing::VerificationError;
use jormungandr_lib::interfaces::Block0ConfigurationError;
use jormungandr_lib::interfaces::FragmentStatus;
use std::path::PathBuf;
use std::time::Duration;
use thor::FragmentSenderError;
use thor::FragmentVerifierError;
use thor::WalletError;
use vit_servicing_station_tests::common::startup::server::ServerBootstrapperError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Interactive(#[from] jortestkit::console::InteractiveCommandError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    ParseTime(#[from] time::error::Parse),
    #[error(transparent)]
    Block0Error(#[from] jormungandr_automation::testing::block0::Block0Error),
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error(transparent)]
    Wallet(#[from] WalletError),
    #[error(transparent)]
    FragmentSender(#[from] FragmentSenderError),
    #[error(transparent)]
    FragmentVerifier(#[from] FragmentVerifierError),
    #[error(transparent)]
    VerificationFailed(#[from] VerificationError),
    #[error(transparent)]
    MonitorResourcesError(#[from] ConsumptionBenchmarkError),
    #[error(transparent)]
    VitStationControllerError(#[from] VitStationControllerError),
    #[error(transparent)]
    WalletProxyError(#[from] WalletProxyError),
    #[error(transparent)]
    TemplateLoadError(#[from] vit_servicing_station_tests::common::data::TemplateLoad),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYamlError(#[from] serde_yaml::Error),
    #[error(transparent)]
    Block0EncodeError(#[from] chain_impl_mockchain::ledger::Error),
    #[error(transparent)]
    ImageReadError(#[from] image::error::ImageError),
    #[error(transparent)]
    MockError(#[from] crate::cli::start::MockError),
    #[error(transparent)]
    ClientRestError(#[from] crate::client::rest::Error),
    #[error(transparent)]
    Block0ConfigurationError(#[from] Block0ConfigurationError),
    #[error(transparent)]
    VitServerBootstrapperError(#[from] ServerBootstrapperError),
    #[error(transparent)]
    VitRestError(#[from] vit_servicing_station_tests::common::clients::RestError),
    #[error(transparent)]
    ChainAddressError(#[from] chain_addr::Error),
    #[error(transparent)]
    ChainBech32Error(#[from] chain_crypto::bech32::Error),
    #[error(transparent)]
    GlobError(#[from] glob::GlobError),
    #[error(transparent)]
    ValgrindError(#[from] valgrind::Error),
    #[error(transparent)]
    ImportError(#[from] crate::cli::import::ImportError),
    #[error(transparent)]
    Validate(#[from] crate::cli::ValidateError),
    #[error(transparent)]
    ControllerError(#[from] hersir::controller::Error),
    #[error(transparent)]
    Block0(#[from] Block0Error),
    #[error(transparent)]
    Builder(#[from] crate::builders::Error),
    #[error(transparent)]
    Certs(#[from] crate::config::certs::Error),
    #[error(transparent)]
    Data(#[from] crate::mode::standard::DataError),
    #[error(transparent)]
    Main(#[from] crate::mode::standard::VitControllerError),
    #[error(transparent)]
    WalletProxyController(#[from] WalletProxyControllerError),
    #[error("Cannot find snapshot file in: {0}")]
    CannotFindSnapshotFile(PathBuf),
    #[error("Cannot find config in: {0}")]
    CannotFindConfig(PathBuf),
    #[error("synchronization for nodes has failed. {}. Timeout was: {} s", info, timeout.as_secs())]
    SyncTimeoutOccurred { info: String, timeout: Duration },
    #[error("{info}")]
    AssertionFailed { info: String },
    #[error(
        "transaction should be 'In Block'. status: {:?}, node: {}",
        status,
        node
    )]
    TransactionNotInBlock {
        node: String,
        status: FragmentStatus,
    },
    #[error("proxy with alias: {alias} not found")]
    ProxyNotFound { alias: String },
    #[error("unknown log level: {0}")]
    UnknownLogLevel(String),
    #[error("environment is down")]
    EnvironmentIsDown,
    #[error("wrong format for snapshot data")]
    SnapshotIntialReadError,
    #[error("no challenge id found for proposal {proposal_id}")]
    NoChallengeIdFound { proposal_id: String },
    #[error("no challenge id: {id} and group: {group} found")]
    NoChallengeIdAndGroupFound { id: String, group: String },
}
