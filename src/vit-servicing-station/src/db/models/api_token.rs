use crate::db::{schema::api_tokens, DB};
use crate::v0::api_token::APIToken;
use diesel::Queryable;

#[allow(dead_code)]
pub struct APITokenData {
    token: APIToken,
    creation_time: String,
    expire_time: String,
}

impl Queryable<api_tokens::SqlType, DB> for APITokenData {
    type Row = (Vec<u8>, String, String);

    fn build(row: Self::Row) -> Self {
        Self {
            token: APIToken::new(row.0),
            creation_time: row.1,
            expire_time: row.2,
        }
    }
}
