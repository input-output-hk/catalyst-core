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
    #[serde(alias = "internalId")]
    // this is used only to retain the original insert order
    pub internal_id: i32,
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
        // 0 -> internal_id
        i32,
        // 1 -> id
        i32,
        // 2 -> challenge_type
        String,
        // 3 -> title
        String,
        // 4 -> description
        String,
        // 5 -> rewards_total
        i64,
        // 6 -> proposers_rewards
        i64,
        // 7 -> fund_id
        i32,
        // 8 -> fund_url
        String,
        // 9 -> challenge_highlights
        Option<String>,
    );

    fn build(row: Self::Row) -> Self {
        Challenge {
            internal_id: row.0,
            id: row.1,
            challenge_type: row.2.parse().unwrap(),
            title: row.3,
            description: row.4,
            rewards_total: row.5,
            proposers_rewards: row.6,
            fund_id: row.7,
            challenge_url: row.8,
            // It should be ensured that the content is valid json
            highlights: row.9.and_then(|v| serde_json::from_str(&v).ok()),
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
            internal_id: 1,
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
