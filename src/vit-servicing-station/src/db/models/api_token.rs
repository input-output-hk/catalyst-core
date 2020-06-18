use crate::db::{schema::api_tokens, DB};
use crate::utils::datetime::unix_timestamp_to_datetime;
use crate::v0::api_token::APIToken;
use chrono::{DateTime, Utc};
use diesel::Queryable;

#[allow(dead_code)]
pub struct APITokenData {
    token: APIToken,
    creation_time: DateTime<Utc>,
    expire_time: DateTime<Utc>,
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
            creation_time: unix_timestamp_to_datetime(row.1),
            expire_time: unix_timestamp_to_datetime(row.2),
        }
    }
}
