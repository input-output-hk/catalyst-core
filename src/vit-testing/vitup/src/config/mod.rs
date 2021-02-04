mod env;
mod initials;

pub use env::VitStartParameters;
pub use initials::Initials;

use chain_impl_mockchain::fee::LinearFee;
use jormungandr_lib::interfaces::{CommitteeIdDef, ConsensusLeaderId, Initial, LinearFeeDef};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataGenerationConfig {
    #[serde(default)]
    pub consensus_leader_ids: Vec<ConsensusLeaderId>,
    #[serde(with = "LinearFeeDef")]
    pub linear_fees: LinearFee,
    #[serde(default)]
    pub committees: Vec<CommitteeIdDef>,
    #[serde(default)]
    pub additions: Vec<Initial>,
    #[serde(flatten)]
    pub params: VitStartParameters,
}

impl Default for DataGenerationConfig {
    fn default() -> Self {
        Self {
            consensus_leader_ids: Vec::new(),
            linear_fees: LinearFee::new(0, 0, 0),
            committees: Vec::new(),
            additions: Vec::new(),
            params: Default::default(),
        }
    }
}
