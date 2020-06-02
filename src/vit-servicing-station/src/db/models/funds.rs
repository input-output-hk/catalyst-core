use crate::db::{models::vote_plan::ChainVoteplan, schema::funds, DB};
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Fund {
    pub fund_name: String,
    pub fund_goal: String,
    pub voting_power_info: String,
    pub rewards_info: String,
    pub fund_start_time: String,
    pub fund_end_time: String,
    pub next_fund_start_time: String,
    pub chain_vote_plans: Vec<ChainVoteplan>,
}

impl Queryable<funds::SqlType, DB> for Fund {
    type Row = (
        // 0 -> fund_name
        String,
        // 1-> fund_goal
        String,
        // 2 -> voting_power_info
        String,
        // 3 -> rewards_info
        String,
        // 4 -> fund_start_time
        String,
        // 5 -> fund_end_time
        String,
        // 6 -> next_fund_start_time
        String,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            fund_name: row.0,
            fund_goal: row.1,
            voting_power_info: row.2,
            rewards_info: row.3,
            fund_start_time: row.4,
            fund_end_time: row.5,
            next_fund_start_time: row.6,
            chain_vote_plans: vec![],
        }
    }
}
