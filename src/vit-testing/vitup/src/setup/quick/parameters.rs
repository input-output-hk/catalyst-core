use crate::setup::initials::Initials;
use chrono::NaiveDateTime;

#[derive(Clone, Debug)]
pub struct QuickVitBackendParameters {
    pub initials: Initials,
    pub vote_start: u64,
    pub vote_tally: u64,
    pub tally_end: u64,
    pub vote_start_timestamp: Option<NaiveDateTime>,
    pub tally_start_timestamp: Option<NaiveDateTime>,
    pub tally_end_timestamp: Option<NaiveDateTime>,
    pub next_vote_start_time: Option<NaiveDateTime>,
    pub proposals: u32,
    pub slot_duration: u8,
    pub slots_per_epoch: u32,
    pub voting_power: u64,
    pub fund_name: String,
    pub private: bool,
}

impl Default for QuickVitBackendParameters {
    fn default() -> Self {
        Self {
            initials: Default::default(),
            vote_start: 1,
            vote_tally: 2,
            tally_end: 3,
            proposals: 100,
            slot_duration: 20,
            slots_per_epoch: 30,
            voting_power: 8000,
            vote_start_timestamp: None,
            tally_start_timestamp: None,
            tally_end_timestamp: None,
            next_vote_start_time: None,
            fund_name: "fund_3".to_owned(),
            private: false,
        }
    }
}
