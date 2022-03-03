use crate::db::models::api_tokens::ApiTokenData;
use crate::db::{
    models::api_tokens as api_token_model,
    schema::{api_tokens, api_tokens::dsl::api_tokens as api_tokens_dsl},
    DbConnection, DbConnectionPool,
};
use crate::v0::api_token::ApiToken;
use crate::v0::errors::HandleError;
use diesel::query_dsl::RunQueryDsl;
use diesel::{ExpressionMethods, Insertable, OptionalExtension, QueryDsl, QueryResult};
use time::{Duration, OffsetDateTime};

pub async fn query_token(
    token: ApiToken,
    pool: &DbConnectionPool,
) -> Result<Option<api_token_model::ApiTokenData>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        query_token_data_by_token(token.as_ref(), &db_conn)
            .map_err(|e| HandleError::InternalError(e.to_string()))
    })
    .await
    .map_err(|_| HandleError::InternalError("Error executing request".to_string()))?
}

/// Insert a token asynchronously. This method is a wrapper over `insert_data_token` that uses the same
/// approach synchronously for a complete formed APITokenData object related to the database model.
pub async fn insert_token(token: &ApiToken, pool: &DbConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;

    let api_token_data = ApiTokenData {
        token: token.clone(),
        creation_time: OffsetDateTime::now_utc().unix_timestamp(),
        expire_time: (OffsetDateTime::now_utc() + Duration::days(365)).unix_timestamp(),
    };

    tokio::task::spawn_blocking(move || {
        insert_token_data(api_token_data, &db_conn)
            .map(|_| ())
            .map_err(|e| HandleError::InternalError(e.to_string()))
    })
    .await
    .map_err(|_| HandleError::InternalError("Error executing request".to_string()))?
}

pub fn query_token_data_by_token(
    raw_token: &[u8],
    db_conn: &DbConnection,
) -> Result<Option<api_token_model::ApiTokenData>, diesel::result::Error> {
    api_tokens_dsl
        .filter(api_tokens::token.eq(raw_token))
        .first::<api_token_model::ApiTokenData>(db_conn)
        .optional()
}

pub fn insert_token_data(token_data: ApiTokenData, db_conn: &DbConnection) -> QueryResult<usize> {
    diesel::insert_into(api_tokens::table)
        .values(token_data.values())
        .execute(db_conn)
}

pub fn batch_insert_token_data(
    tokens_data: &[ApiTokenData],
    db_conn: &DbConnection,
) -> QueryResult<usize> {
    diesel::insert_into(api_tokens::table)
        .values(
            tokens_data
                .iter()
                .map(|t| t.clone().values())
                .collect::<Vec<_>>(),
        )
        .execute(db_conn)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::{
        load_db_connection_pool, migrations as db_testing, models::api_tokens::ApiTokenData,
        DbConnectionPool,
    };

    #[tokio::test]
    async fn api_token_insert_and_retrieve() {
        // initialize db
        let pool: DbConnectionPool = load_db_connection_pool("").unwrap();
        db_testing::initialize_db_with_migration(&pool.get().unwrap());

        // checks
        let token = ApiToken::new(b"foo_bar_zen".to_vec());
        insert_token(&token, &pool).await.unwrap();
        let token_data: ApiTokenData = query_token(token.clone(), &pool).await.unwrap().unwrap();
        assert_eq!(token_data.token, token);
    }
}
