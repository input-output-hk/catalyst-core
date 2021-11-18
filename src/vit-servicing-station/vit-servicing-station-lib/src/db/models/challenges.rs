use crate::db::models::proposals::ChallengeType;
use crate::db::{schema::challenges, Db};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ChallengeHighlights {
    pub sponsor: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub id: i32,
    #[serde(alias = "challengeType")]
    pub challenge_type: ChallengeType,
    pub title: String,
    pub description: String,
    #[serde(alias = "rewardsTotal")]
    pub rewards_total: i64,
    #[serde(alias = "proposersRewards")]
    pub proposers_rewards: i64,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "challengeUrl")]
    pub challenge_url: String,
    pub highlights: Option<ChallengeHighlights>,
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
        // 4 -> proposers_rewards
        i64,
        // 5 -> fund_id
        i32,
        // 6 -> fund_url
        String,
        // 7 -> challenge_highlights
        Option<String>,
    );

    fn build(row: Self::Row) -> Self {
        Challenge {
            id: row.0,
            challenge_type: row.1.parse().unwrap(),
            title: row.2,
            description: row.3,
            rewards_total: row.4,
            proposers_rewards: row.5,
            fund_id: row.6,
            challenge_url: row.7,
            // It should be ensured that the content is valid json
            highlights: row.8.and_then(|v| serde_json::from_str(&v).ok()),
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
        diesel::dsl::Eq<challenges::proposers_rewards, i64>,
        diesel::dsl::Eq<challenges::fund_id, i32>,
        diesel::dsl::Eq<challenges::challenge_url, String>,
        diesel::dsl::Eq<challenges::highlights, Option<String>>,
    );

    fn values(self) -> Self::Values {
        (
            challenges::id.eq(self.id),
            challenges::challenge_type.eq(self.challenge_type.to_string()),
            challenges::title.eq(self.title),
            challenges::description.eq(self.description),
            challenges::rewards_total.eq(self.rewards_total),
            challenges::proposers_rewards.eq(self.proposers_rewards),
            challenges::fund_id.eq(self.fund_id),
            challenges::challenge_url.eq(self.challenge_url),
            // This should always be a valid json
            challenges::highlights.eq(serde_json::to_string(&self.highlights).ok()),
        )
    }
}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::db::DbConnectionPool;
    use diesel::RunQueryDsl;

    pub fn get_test_challenge_with_fund_id(fund_id: i32) -> Challenge {
        const CHALLENGE_ID: i32 = 9001;
        const REWARDS_TOTAL: i64 = 100500;
        Challenge {
            id: CHALLENGE_ID,
            challenge_type: ChallengeType::CommunityChoice,
            title: "challenge title".to_string(),
            description: "challenge description".to_string(),
            rewards_total: REWARDS_TOTAL,
            proposers_rewards: REWARDS_TOTAL,
            fund_id,
            challenge_url: "http://example.com/".to_string(),
            highlights: None,
        }
    }

    pub fn populate_db_with_challenge(challenge: &Challenge, pool: &DbConnectionPool) {
        let connection = pool.get().unwrap();

        diesel::insert_into(challenges::table)
            .values(challenge.clone().values())
            .execute(&connection)
            .unwrap();
    }
}
