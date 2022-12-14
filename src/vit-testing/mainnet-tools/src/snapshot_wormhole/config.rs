use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use snapshot_lib::Fraction;

/// Configuration. It contains 3 parts snapshot-service connection, servicing station configuration
/// and parameters of single import (as we need to set e.g. tag under which our snapshot will be put)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// Snapshot service related configuration
    pub snapshot_service: SnapshotService,
    /// Servicing service related configuration
    pub servicing_station: ServicingStation,
    /// Import parameters
    pub(crate) parameters: Parameters,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Parameters {
    /// Tag under which snapshot content will be available
    #[serde(default = "default_latest")]
    pub tag: String,
    /// Minimum lovelace which is required to participate in voting
    pub min_stake_threshold: Value,
    /// Maximum percentage of voting power before capping
    pub voting_power_cap: Fraction,
    /// Name of direct registration holders
    pub direct_voters_group: Option<String>,
    /// Name of delegated registrations holders (representatives)
    pub representatives_group: Option<String>,
}

fn default_latest() -> String {
    "latest".to_string()
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
