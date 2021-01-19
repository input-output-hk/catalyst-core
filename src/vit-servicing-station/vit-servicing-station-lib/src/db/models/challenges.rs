use crate::db::{schema::challenges, DB};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub rewards_total: i64,
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
