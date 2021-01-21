use crate::db::{
    models::{challenges::Challenge, voteplans::Voteplan},
    schema::funds,
    DB,
};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Fund {
    #[serde(default = "Default::default")]
    pub id: i32,
    #[serde(alias = "fundName")]
    pub fund_name: String,
    #[serde(alias = "fundGoal")]
    pub fund_goal: String,
    #[serde(alias = "votingPowerInfo")]
    pub voting_power_info: String,
    #[serde(alias = "votingPowerThreshold")]
    pub voting_power_threshold: i64,
    #[serde(alias = "rewardsInfo")]
    pub rewards_info: String,
    #[serde(alias = "fundStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub fund_start_time: i64,
    #[serde(alias = "fundEndTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub fund_end_time: i64,
    #[serde(alias = "nextFundStartTime")]
    #[serde(serialize_with = "crate::utils::serde::serialize_unix_timestamp_as_rfc3339")]
    #[serde(deserialize_with = "crate::utils::serde::deserialize_unix_timestamp_from_rfc3339")]
    pub next_fund_start_time: i64,
    #[serde(alias = "chainVotePlans", default = "Vec::new")]
    pub chain_vote_plans: Vec<Voteplan>,
    #[serde(default = "Vec::new")]
    pub challenges: Vec<Challenge>,
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
        // 4 -> voting_power_threshold
        i64,
        // 5 -> rewards_info
        String,
        // 6 -> fund_start_time
        i64,
        // 7 -> fund_end_time
        i64,
        // 8 -> next_fund_start_time
        i64,
    );

    fn build(row: Self::Row) -> Self {
        Fund {
            id: row.0,
            fund_name: row.1,
            fund_goal: row.2,
            voting_power_info: row.3,
            voting_power_threshold: row.4,
            rewards_info: row.5,
            fund_start_time: row.6,
            fund_end_time: row.7,
            next_fund_start_time: row.8,
            chain_vote_plans: vec![],
            challenges: vec![],
        }
    }
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<funds::table> for Fund {
    type Values = (
        Option<diesel::dsl::Eq<funds::id, i32>>,
        diesel::dsl::Eq<funds::fund_name, String>,
        diesel::dsl::Eq<funds::fund_goal, String>,
        diesel::dsl::Eq<funds::voting_power_info, String>,
        diesel::dsl::Eq<funds::voting_power_threshold, i64>,
        diesel::dsl::Eq<funds::rewards_info, String>,
        diesel::dsl::Eq<funds::fund_start_time, i64>,
        diesel::dsl::Eq<funds::fund_end_time, i64>,
        diesel::dsl::Eq<funds::next_fund_start_time, i64>,
    );

    fn values(self) -> Self::Values {
        let id_item = if self.id == 0 {
            None
        } else {
            Some(funds::id.eq(self.id))
        };
        (
            id_item,
            funds::fund_name.eq(self.fund_name),
            funds::fund_goal.eq(self.fund_goal),
            funds::voting_power_info.eq(self.voting_power_info),
            funds::voting_power_threshold.eq(self.voting_power_threshold),
            funds::rewards_info.eq(self.rewards_info),
            funds::fund_start_time.eq(self.fund_start_time),
            funds::fund_end_time.eq(self.fund_end_time),
            funds::next_fund_start_time.eq(self.next_fund_start_time),
        )
    }
}

#[cfg(test)]
pub mod test {
    use crate::db::{
        models::{
            challenges::test as challenges_testing, funds::Fund,
            voteplans::test as voteplans_testing,
        },
        schema::funds,
        DBConnectionPool,
    };

    use chrono::Utc;
    use diesel::{ExpressionMethods, RunQueryDsl};

    pub fn get_test_fund() -> Fund {
        const FUND_ID: i32 = 42;
        Fund {
            id: FUND_ID,
            fund_name: "hey oh let's go".to_string(),
            fund_goal: "test this endpoint".to_string(),
            voting_power_info: ">9000".to_string(),
            voting_power_threshold: 100,
            rewards_info: "not much".to_string(),
            fund_start_time: Utc::now().timestamp(),
            fund_end_time: Utc::now().timestamp(),
            next_fund_start_time: Utc::now().timestamp(),
            chain_vote_plans: vec![voteplans_testing::get_test_voteplan_with_fund_id(FUND_ID)],
            challenges: vec![challenges_testing::get_test_challenge_with_fund_id(FUND_ID)],
        }
    }

    pub fn populate_db_with_fund(fund: &Fund, pool: &DBConnectionPool) {
        let values = (
            funds::id.eq(fund.id),
            funds::fund_name.eq(fund.fund_name.clone()),
            funds::fund_goal.eq(fund.fund_goal.clone()),
            funds::voting_power_info.eq(fund.voting_power_info.clone()),
            funds::voting_power_threshold.eq(fund.voting_power_threshold),
            funds::rewards_info.eq(fund.rewards_info.clone()),
            funds::fund_start_time.eq(fund.fund_start_time),
            funds::fund_end_time.eq(fund.fund_end_time),
            funds::next_fund_start_time.eq(fund.next_fund_start_time),
        );

        // Warning! mind this scope: r2d2 pooled connection behaviour depend of the scope. Looks like
        // if the connection is not out of scope, when giving the reference to the next function
        // call below it creates a wrong connection (where there are not tables even if they were loaded).
        {
            let connection = pool.get().unwrap();
            diesel::insert_into(funds::table)
                .values(values)
                .execute(&connection)
                .unwrap();
        }

        for voteplan in &fund.chain_vote_plans {
            voteplans_testing::populate_db_with_voteplan(voteplan, pool);
        }

        for challenge in &fund.challenges {
            challenges_testing::populate_db_with_challenge(challenge, pool);
        }
    }
}
