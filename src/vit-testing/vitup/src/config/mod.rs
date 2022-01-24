mod env;
mod initials;
pub mod mode;
mod vote_time;

pub use env::VitStartParameters;
pub use initials::{Initial as InitialEntry, Initials};
pub use vote_time::{VoteBlockchainTime, VoteTime, FORMAT as VOTE_TIME_FORMAT};

use crate::builders::utils::io::read_initials;
use crate::Result;
use chain_impl_mockchain::fee::LinearFee;
use jormungandr_lib::interfaces::{CommitteeIdDef, ConsensusLeaderId, LinearFeeDef};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DataGenerationConfig {
    #[serde(default)]
    pub consensus_leader_ids: Vec<ConsensusLeaderId>,
    #[serde(with = "LinearFeeDef")]
    pub linear_fees: LinearFee,
    #[serde(default)]
    pub committees: Vec<CommitteeIdDef>,
    #[serde(flatten)]
    pub params: VitStartParameters,
}

impl Default for DataGenerationConfig {
    fn default() -> Self {
        Self {
            consensus_leader_ids: Vec::new(),
            linear_fees: LinearFee::new(0, 0, 0),
            committees: Vec::new(),
            params: Default::default(),
        }
    }
}

impl DataGenerationConfig {
    pub fn extend_from_initials_file<P: AsRef<Path>>(&mut self, snapshot: P) -> Result<()> {
        self.params
            .initials
            .extend_from_external(read_initials(snapshot)?);
        Ok(())
    }
}

pub fn read_params<P: AsRef<Path>>(params: P) -> Result<VitStartParameters> {
    let contents = std::fs::read_to_string(&params)?;
    serde_yaml::from_str(&contents).map_err(Into::into)
}
