use super::initials::Initials;
use crate::config::VoteTime;
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
    pub vote_time: VoteTime,
    pub next_vote_start_time: Option<NaiveDateTime>,
    pub refresh_time: Option<NaiveDateTime>,
    pub proposals: u32,
    pub challenges: usize,
    pub reviews: usize,
    pub slot_duration: u8,
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
        match self.vote_time {
            VoteTime::Blockchain(blockchain) => {
                let duration_as_secs = (blockchain.tally_end - blockchain.vote_start) as u64
                    * self.slot_duration as u64
                    * (blockchain.slots_per_epoch - 1) as u64;

                Duration::from_secs(duration_as_secs)
            }
            VoteTime::Real {
                vote_start_timestamp,
                tally_start_timestamp,
                tally_end_timestamp: _,
                find_best_match: _,
            } => Duration::from_secs(
                (tally_start_timestamp - vote_start_timestamp).num_seconds() as u64
            ),
        }
    }
}

impl Default for VitStartParameters {
    fn default() -> Self {
        Self {
            protocol: Protocol::Http,
            initials: Default::default(),
            vote_time: Default::default(),
            proposals: 100,
            challenges: 4,
            reviews: 1,
            slot_duration: 20,
            voting_power: 8000,
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
