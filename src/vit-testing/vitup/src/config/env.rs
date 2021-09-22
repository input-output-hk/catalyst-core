use super::initials::Initials;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use valgrind::Protocol;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VitStartParameters {
    #[serde(default)]
    pub initials: Initials,
    #[serde(default = "Protocol::http")]
    pub protocol: Protocol,
    pub vote_start: u64,
    pub vote_tally: u64,
    pub tally_end: u64,
    pub vote_start_timestamp: Option<NaiveDateTime>,
    pub tally_start_timestamp: Option<NaiveDateTime>,
    pub tally_end_timestamp: Option<NaiveDateTime>,
    pub next_vote_start_time: Option<NaiveDateTime>,
    pub refresh_time: Option<NaiveDateTime>,
    pub proposals: u32,
    pub challenges: usize,
    pub reviews: usize,
    pub slot_duration: u8,
    pub slots_per_epoch: u32,
    pub block_content_max_size: u32,
    pub voting_power: u64,
    pub fund_name: String,
    pub fund_id: i32,
    pub private: bool,
    pub version: String,
    pub tx_max_expiry_epochs: Option<u8>,
}

impl VitStartParameters {
    pub fn calculate_vote_duration(&self) -> Duration {
        let duration_as_secs = (self.vote_tally - self.vote_start)
            * self.slot_duration as u64
            * (self.slots_per_epoch - 1) as u64;

        Duration::from_secs(duration_as_secs)
    }
}

impl Default for VitStartParameters {
    fn default() -> Self {
        Self {
            protocol: Protocol::Http,
            initials: Default::default(),
            vote_start: 1,
            vote_tally: 2,
            tally_end: 3,
            proposals: 100,
            challenges: 4,
            reviews: 1,
            slot_duration: 20,
            slots_per_epoch: 30,
            voting_power: 8000,
            vote_start_timestamp: None,
            tally_start_timestamp: None,
            tally_end_timestamp: None,
            next_vote_start_time: None,
            refresh_time: None,
            block_content_max_size: 102400,
            fund_name: "fund_3".to_owned(),
            private: false,
            fund_id: 1,
            version: "2.0".to_string(),
            tx_max_expiry_epochs: Some(2),
        }
    }
}
