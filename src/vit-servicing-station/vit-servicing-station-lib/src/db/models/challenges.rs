use crate::db::{schema::challenges, DB};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
}

impl Queryable<challenges::SqlType, DB> for Challenge {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> title
        String,
        // 2 -> description
        String,
        // 3 -> rewards_total
        i64,
        // 4 -> fund_id
        i32,
    );

    fn build(row: Self::Row) -> Self {
        Challenge {
            id: row.0,
            title: row.1,
            description: row.2,
            rewards_total: row.3,
            fund_id: row.4,
        }
    }
}

impl Insertable<challenges::table> for Challenge {
    #[allow(clippy::type_complexity)]
    type Values = (
        diesel::dsl::Eq<challenges::id, i32>,
        diesel::dsl::Eq<challenges::title, String>,
        diesel::dsl::Eq<challenges::description, String>,
        diesel::dsl::Eq<challenges::rewards_total, i64>,
        diesel::dsl::Eq<challenges::fund_id, i32>,
    );

    fn values(self) -> Self::Values {
        (
            challenges::id.eq(self.id),
            challenges::title.eq(self.title),
            challenges::description.eq(self.description),
            challenges::rewards_total.eq(self.rewards_total),
            challenges::fund_id.eq(self.fund_id),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DBConnectionPool;
    use diesel::RunQueryDsl;

    pub fn get_test_challenge_with_fund_id(fund_id: i32) -> Challenge {
        const CHALLENGE_ID: i32 = 9001;
        const REWARDS_TOTAL: i64 = 100500;
        Challenge {
            id: CHALLENGE_ID,
            title: "challenge title".to_string(),
            description: "challenge description".to_string(),
            rewards_total: REWARDS_TOTAL,
            fund_id,
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
        );

        diesel::insert_into(challenges::table)
            .values(values)
            .execute(&connection)
            .unwrap();
    }
}
