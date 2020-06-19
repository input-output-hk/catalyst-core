use crate::db::{models::voteplans::Voteplan, schema::funds, DB};
use crate::utils::datetime::unix_timestamp_to_datetime;
use chrono::{DateTime, Utc};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Eq)]
pub struct Fund {
    pub id: i32,
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_info: String,
    pub rewards_info: String,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub fund_start_time: DateTime<Utc>,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub fund_end_time: DateTime<Utc>,
    #[serde(serialize_with = "crate::utils::serde::serialize_datetime_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_datetime_from_rfc3339")]
    pub next_fund_start_time: DateTime<Utc>,
    pub chain_vote_plans: Vec<Voteplan>,
}

impl Queryable<funds::SqlType, DB> for Fund {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> fund_name
        String,
        // 2 -> fund_goal
        String,
        // 3 -> voting_power_info
        String,
        // 4 -> rewards_info
        String,
        // 5 -> fund_start_time
        i64,
        // 6 -> fund_end_time
        i64,
        // 7 -> next_fund_start_time
        i64,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            id: row.0,
            fund_name: row.1,
            fund_goal: row.2,
            voting_power_info: row.3,
            rewards_info: row.4,
            fund_start_time: unix_timestamp_to_datetime(row.5),
            fund_end_time: unix_timestamp_to_datetime(row.6),
            next_fund_start_time: unix_timestamp_to_datetime(row.7),
            chain_vote_plans: vec![],
        }
    }
}

// This implementation is needed because there is an issue when serializing/deserializing DataTime
// in where for the same data it is deserialized with different subseconds representation.
impl PartialEq for Fund {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.fund_name == other.fund_name
            && self.fund_goal == other.fund_goal
            && self.voting_power_info == other.voting_power_info
            && self.rewards_info == other.rewards_info
            && self.fund_start_time.timestamp() == other.fund_start_time.timestamp()
            && self.fund_end_time.timestamp() == other.fund_end_time.timestamp()
            && self.next_fund_start_time.timestamp() == other.next_fund_start_time.timestamp()
            && self.chain_vote_plans == other.chain_vote_plans
    }
}
