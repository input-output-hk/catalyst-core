use crate::db::models::proposals::ChallengeType;
use crate::db::{schema::challenges, Db};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeUrl")]
    pub challenge_url: String,
}

impl Queryable<challenges::SqlType, Db> for Challenge {
    type Row = (
        // 0 -> id
        i32,
        // 1 -> challenge_type
        String,
        // 1 -> title
        String,
        // 2 -> description
        String,
        // 3 -> rewards_total
        i64,
        // 4 -> fund_id
        i32,
        // 5 -> fund_url
        String,
    );

    fn build(row: Self::Row) -> Self {
        Challenge {
            id: row.0,
            challenge_type: row.1.parse().unwrap(),
            title: row.2,
            description: row.3,
            rewards_total: row.4,
            fund_id: row.5,
            challenge_url: row.6,
        }
    }
}

impl Insertable<challenges::table> for Challenge {
    #[allow(clippy::type_complexity)]
    type Values = (
        diesel::dsl::Eq<challenges::id, i32>,
        diesel::dsl::Eq<challenges::challenge_type, String>,
        diesel::dsl::Eq<challenges::title, String>,
        diesel::dsl::Eq<challenges::description, String>,
        diesel::dsl::Eq<challenges::rewards_total, i64>,
        diesel::dsl::Eq<challenges::fund_id, i32>,
        diesel::dsl::Eq<challenges::challenge_url, String>,
    );

    fn values(self) -> Self::Values {
        (
            challenges::id.eq(self.id),
            challenges::challenge_type.eq(self.challenge_type.to_string()),
            challenges::title.eq(self.title),
            challenges::description.eq(self.description),
            challenges::rewards_total.eq(self.rewards_total),
            challenges::fund_id.eq(self.fund_id),
            challenges::challenge_url.eq(self.challenge_url),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DbConnectionPool;
    use diesel::{ExpressionMethods, RunQueryDsl};

    pub fn get_test_challenge_with_fund_id(fund_id: i32) -> Challenge {
        const CHALLENGE_ID: i32 = 9001;
        const REWARDS_TOTAL: i64 = 100500;
        Challenge {
            id: CHALLENGE_ID,
            challenge_type: ChallengeType::CommunityChoice,
            title: "challenge title".to_string(),
            description: "challenge description".to_string(),
            rewards_total: REWARDS_TOTAL,
            fund_id,
            challenge_url: "http://example.com/".to_string(),
        }
    }

    pub fn populate_db_with_challenge(challenge: &Challenge, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();

        let values = (
            challenges::id.eq(challenge.id),
            challenges::challenge_type.eq(challenge.challenge_type.to_string()),
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
