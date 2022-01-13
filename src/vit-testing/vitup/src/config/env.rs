use super::initials::{Initial, Initials};
use crate::builders::{default_next_snapshot_date, default_next_vote_date, default_snapshot_date};
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
    pub block0_time: Option<NaiveDateTime>,
    pub next_vote_start_time: NaiveDateTime,
    pub snapshot_time: NaiveDateTime,
    pub next_snapshot_time: NaiveDateTime,
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
            initials: Initials(vec![Initial::AboveThreshold {
                above_threshold: 10,
                pin: "1234".to_string(),
            }]),
            vote_time: Default::default(),
            proposals: 100,
            challenges: 9,
            reviews: 3,
            slot_duration: 20,
            voting_power: 8000,
            block0_time: None,
            snapshot_time: default_snapshot_date(),
            next_snapshot_time: default_next_snapshot_date(),
            next_vote_start_time: default_next_vote_date(),
            block_content_max_size: 102400,
            fund_name: "fund_3".to_owned(),
            private: false,
            fund_id: 1,
            version: "2.0".to_string(),
            tx_max_expiry_epochs: Some(2),
        }
    }
}
