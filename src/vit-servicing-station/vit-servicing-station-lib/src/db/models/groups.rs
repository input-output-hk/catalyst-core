use crate::db::schema::groups;
use diesel::{ExpressionMethods, Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Queryable, PartialOrd, Ord)]
pub struct Group {
    #[serde(alias = "fundId")]
    pub fund_id: i32,
    #[serde(alias = "tokenId")]
    pub token_identifier: String,
    #[serde(alias = "groupId")]
    pub group_id: String,
}

impl Insertable<groups::table> for Group {
    type Values = (
        diesel::dsl::Eq<groups::fund_id, i32>,
        diesel::dsl::Eq<groups::token_identifier, String>,
        diesel::dsl::Eq<groups::group_id, String>,
    );

    fn values(self) -> Self::Values {
        (
            groups::fund_id.eq(self.fund_id),
            groups::token_identifier.eq(self.token_identifier),
            groups::group_id.eq(self.group_id),
        )
    }
}
