use crate::network::wallet_state::Actor;
use jormungandr_lib::interfaces::Value;
use serde::{Deserialize, Serialize};
use snapshot_lib::{Dreps, Fraction};

/// Root struct for defining snapshot template
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Initials {
    /// snapshot content
    pub content: Vec<Actor>,
    /// parameters
    #[serde(default = "Parameters::default")]
    pub parameters: Parameters,
}

/// Snapshot
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
    /// dreps information
    pub dreps: Option<Dreps>,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters {
            tag: default_latest(),
            min_stake_threshold: 500.into(),
            voting_power_cap: Fraction::new(1u64, 2u64),
            direct_voters_group: None,
            representatives_group: None,
            dreps: None,
        }
    }
}

fn default_latest() -> String {
    "latest".to_string()
}
