use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use snapshot_lib::Fraction;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub snapshot_service: SnapshotServiceConfig,
    pub servicing_station: ServicingStationConfig,
    pub(crate) parameters: ParametersConfig,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ParametersConfig {
    #[serde(default = "default_latest")]
    pub tag: String,
    pub min_stake_threshold: Value,
    pub voting_power_cap: Fraction,
    pub direct_voters_group: Option<String>,
    pub representatives_group: Option<String>,
}

fn default_latest() -> String {
    "latest".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SnapshotServiceConfig {
    pub token: Option<String>,
    pub address: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServicingStationConfig {
    pub token: Option<String>,
    pub address: String,
}
