use crate::db::{schema::api_tokens, Db};
use crate::v0::api_token::ApiToken;
use diesel::{ExpressionMethods, Insertable, Queryable};

#[derive(Debug, Clone)]
pub struct ApiTokenData {
    pub token: ApiToken,
    pub creation_time: i64,
    pub expire_time: i64,
}

impl Queryable<api_tokens::SqlType, Db> for ApiTokenData {
    type Row = (
        // 0 -> token
        Vec<u8>,
        // 1 -> creation_time
        i64,
        // 2-> expire_time
        i64,
    );

    fn build(row: Self::Row) -> Self {
        Self {
            token: ApiToken::new(row.0),
            creation_time: row.1,
            expire_time: row.2,
        }
    }
}

// This warning is disabled here. Values is only referenced as a type here. It should be ok not to
// split the types definitions.
#[allow(clippy::type_complexity)]
impl Insertable<api_tokens::table> for ApiTokenData {
    type Values = (
        diesel::dsl::Eq<api_tokens::token, Vec<u8>>,
        diesel::dsl::Eq<api_tokens::creation_time, i64>,
        diesel::dsl::Eq<api_tokens::expire_time, i64>,
    );

    fn values(self) -> Self::Values {
        (
            api_tokens::token.eq(self.token.as_ref().to_vec()),
            api_tokens::creation_time.eq(self.creation_time),
            api_tokens::expire_time.eq(self.expire_time),
        )
    }
}
