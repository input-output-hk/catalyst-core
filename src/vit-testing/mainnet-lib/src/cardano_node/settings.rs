use crate::CARDANO_MAINNET_SLOTS_PER_EPOCH;
use chain_impl_mockchain::testing::TestGen;
use std::time::SystemTime;

/// Cardano node mock settings
#[derive(Clone, Debug)]
pub struct Settings {
    /// block0 hash
    pub block0_hash: String,
    /// block0 time
    pub block0_time: SystemTime,
    /// slot duration
    pub slot_duration: u32,
    /// slots per epoch
    pub slots_per_epoch: u64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            block0_hash: TestGen::hash().to_string(),
            block0_time: SystemTime::now(),
            slot_duration: 1,
            slots_per_epoch: CARDANO_MAINNET_SLOTS_PER_EPOCH,
        }
    }
}
