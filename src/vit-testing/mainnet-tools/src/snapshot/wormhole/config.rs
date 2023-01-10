use mainnet_lib::SnapshotParameters;
use serde::{Deserialize, Serialize};

/// Configuration. It contains 3 parts snapshot-service connection, servicing station configuration
/// and parameters of single import (as we need to set e.g. tag under which our snapshot will be put)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Snapshot service related configuration
    pub snapshot_service: SnapshotService,
    /// Servicing service related configuration
    pub servicing_station: ServicingStation,
    /// Import parameters
    pub(crate) parameters: SnapshotParameters,
}

/// Snapshot service related config
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotService {
    /// Access token
    pub token: Option<String>,
    /// Address. In format: 'https://{host}:{port}'
    pub address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServicingStation {
    /// Access token
    pub token: Option<String>,
    /// Address. In format: 'http[s]://{host}:{port}'
    pub address: String,
}
