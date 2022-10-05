use chain_impl_mockchain::fee::LinearFee;
use jormungandr_lib::interfaces::{CommitteeIdDef, ConsensusLeaderId, LinearFeeDef};
use jormungandr_lib::time::SecondsSinceUnixEpoch;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Blockchain {
    #[serde(default)]
    pub slot_duration: u8,
    #[serde(default)]
    pub block_content_max_size: u32,
    #[serde(with = "time::serde::rfc3339", default = "default_block0_time")]
    pub block0_time: OffsetDateTime,
    #[serde(default)]
    pub tx_max_expiry_epochs: Option<u8>,
    #[serde(default)]
    pub consensus_leader_ids: Vec<ConsensusLeaderId>,
    #[serde(with = "LinearFeeDef")]
    pub linear_fees: LinearFee,
    #[serde(default)]
    pub committees: Vec<CommitteeIdDef>,
}

impl Default for Blockchain {
    fn default() -> Self {
        Self {
            slot_duration: 20,
            block0_time: default_block0_time(),
            block_content_max_size: 102400,
            tx_max_expiry_epochs: Some(2),
            consensus_leader_ids: Vec::new(),
            linear_fees: LinearFee::new(0, 0, 0),
            committees: Vec::new(),
        }
    }
}
impl Blockchain {
    pub fn block0_date_as_unix(&self) -> SecondsSinceUnixEpoch {
        SecondsSinceUnixEpoch::from_secs(self.block0_time.unix_timestamp() as u64)
    }
}

fn default_block0_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}
