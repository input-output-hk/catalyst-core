#![allow(clippy::extra_unused_lifetimes)] // derive(Insertable) has a bug, so this is needed for
                                          // clippy to pass

use crate::db::schema::goals;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Clone, Debug, PartialEq, Eq)]
#[diesel(table_name = goals)]
pub struct Goal {
    pub id: i32,
    #[serde(alias = "goalName")]
    pub goal_name: String,
    #[serde(alias = "fundId")]
    pub fund_id: i32,
}

#[derive(Deserialize, Insertable, Clone, Debug)]
#[table_name = "goals"]
pub struct InsertGoal {
    pub goal_name: String,
    pub fund_id: i32,
}

impl From<&Goal> for InsertGoal {
    fn from(g: &Goal) -> Self {
        Self {
            goal_name: g.goal_name.clone(),
            fund_id: g.fund_id,
        }
    }
}
