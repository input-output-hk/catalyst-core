use crate::db::{schema::api_tokens, DB};
use crate::v0::api_token::APIToken;
use diesel::Queryable;

pub struct APITokenData {
    pub token: APIToken,
    pub creation_time: i64,
    pub expire_time: i64,
}

impl Queryable<api_tokens::SqlType, DB> for APITokenData {
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
            token: APIToken::new(row.0),
            creation_time: row.1,
            expire_time: row.2,
        }
    }
}
