use super::initials::Initials;
use chrono::NaiveDateTime;
use iapyx::Protocol;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VitStartParameters {
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
    pub proposals: u32,
    pub challenges: usize,
    pub slot_duration: u8,
    pub slots_per_epoch: u32,
    pub voting_power: u64,
    pub fund_name: String,
    pub fund_id: i32,
    pub private: bool,
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
            slot_duration: 20,
            slots_per_epoch: 30,
            voting_power: 8000,
            vote_start_timestamp: None,
            tally_start_timestamp: None,
            tally_end_timestamp: None,
            next_vote_start_time: None,
            fund_name: "fund_3".to_owned(),
            private: false,
            fund_id: 1,
        }
    }
}
