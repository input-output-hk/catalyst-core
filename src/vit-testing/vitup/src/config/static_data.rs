use crate::builders::{default_next_snapshot_date, default_next_vote_date, default_snapshot_date};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticData {
    pub options: Vec<String>,
    pub next_vote_start_time: NaiveDateTime,
    pub snapshot_time: NaiveDateTime,
    pub next_snapshot_time: NaiveDateTime,
    pub proposals: u32,
    pub challenges: usize,
    pub reviews: usize,
    pub voting_power: u64,
    pub fund_name: String,
    pub fund_id: i32,
}

impl Default for StaticData {
    fn default() -> Self {
        Self {
            proposals: 100,
            challenges: 9,
            reviews: 3,
            voting_power: 8000,
            snapshot_time: default_snapshot_date(),
            next_snapshot_time: default_next_snapshot_date(),
            next_vote_start_time: default_next_vote_date(),
            options: Vec::new(),
            fund_name: "fund_3".to_owned(),
            fund_id: 1,
        }
    }
}
