use crate::db::{models::voteplans::Voteplan, schema::funds, DB};
use crate::utils::datetime::unix_timestamp_to_datetime;
use chrono::{DateTime, Utc};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fund {
    pub id: i32,
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_info: String,
    pub rewards_info: String,
    pub fund_start_time: DateTime<Utc>,
    pub fund_end_time: DateTime<Utc>,
    pub next_fund_start_time: DateTime<Utc>,
    pub chain_vote_plans: Vec<Voteplan>,
}

impl Queryable<funds::SqlType, DB> for Fund {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> fund_name
        String,
        // 2-> fund_goal
        String,
        // 3 -> voting_power_info
        String,
        // 4 -> rewards_info
        String,
        // 5 -> fund_start_time
        u64,
        // 6 -> fund_end_time
        u64,
        // 7 -> next_fund_start_time
        u64,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            id: row.0,
            fund_name: row.1,
            fund_goal: row.2,
            voting_power_info: row.3,
            rewards_info: row.4,
            fund_start_time: unix_timestamp_to_datetime(row.5 as i64),
            fund_end_time: unix_timestamp_to_datetime(row.6 as i64),
            next_fund_start_time: unix_timestamp_to_datetime(row.7 as i64),
            chain_vote_plans: vec![],
        }
    }
}
