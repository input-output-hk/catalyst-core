use crate::db::schema::challenges;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable, Insertable)]
pub struct Challenge {
    pub id: i32,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeUrl")]
    pub challenge_url: String,
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DBConnectionPool;
    use diesel::{ExpressionMethods, RunQueryDsl};

    pub fn get_test_challenge_with_fund_id(fund_id: i32) -> Challenge {
        const CHALLENGE_ID: i32 = 9001;
        const REWARDS_TOTAL: i64 = 100500;
        Challenge {
            id: CHALLENGE_ID,
            title: "challenge title".to_string(),
            description: "challenge description".to_string(),
            rewards_total: REWARDS_TOTAL,
            fund_id,
            challenge_url: "http://example.com/".to_string(),
        }
    }

    pub fn populate_db_with_challenge(challenge: &Challenge, pool: &DBConnectionPool) {
        let connection = pool.get().unwrap();

        let values = (
            challenges::id.eq(challenge.id),
            challenges::title.eq(challenge.title.clone()),
            challenges::description.eq(challenge.description.clone()),
            challenges::rewards_total.eq(challenge.rewards_total),
            challenges::fund_id.eq(challenge.fund_id),
            challenges::challenge_url.eq(challenge.challenge_url.clone()),
        );

        diesel::insert_into(challenges::table)
            .values(values)
            .execute(&connection)
            .unwrap();
    }
}
