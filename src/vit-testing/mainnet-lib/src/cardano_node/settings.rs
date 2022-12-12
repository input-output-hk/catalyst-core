use chain_impl_mockchain::testing::TestGen;
use std::time::SystemTime;


#[derive(Clone,Debug)]
pub struct Settings {
    pub block0_hash: String,
    pub block0_time: SystemTime,
    pub slot_duration: u32,
    pub slots_per_epoch: u32
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            block0_hash: TestGen::hash().to_string(),
            block0_time: SystemTime::now(),
            slot_duration: 1,
            slots_per_epoch: 43200
        }
    }
}