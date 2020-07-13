use crate::db::{
    models::api_tokens as api_token_model,
    schema::{api_tokens, api_tokens::dsl::api_tokens as api_tokens_dsl},
    DBConnectionPool,
};
use crate::v0::api_token::APIToken;
use crate::v0::errors::HandleError;
use chrono::{Duration, Utc};
use diesel::query_dsl::RunQueryDsl;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};

pub async fn query_token(
    token: APIToken,
    pool: &DBConnectionPool,
) -> Result<Option<api_token_model::APITokenData>, HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    tokio::task::spawn_blocking(move || {
        api_tokens_dsl
            .filter(api_tokens::token.eq(token.as_ref()))
            .first::<api_token_model::APITokenData>(&db_conn)
            .optional()
            .map_err(|e| HandleError::InternalError(e.to_string()))
    })
    .await
    .map_err(|_| HandleError::InternalError("Error executing request".to_string()))?
}

pub async fn insert_token(token: &APIToken, pool: &DBConnectionPool) -> Result<(), HandleError> {
    let db_conn = pool.get().map_err(HandleError::DatabaseError)?;
    let values = (
        api_tokens::token.eq(token.as_ref().to_vec()),
        api_tokens::creation_time.eq(Utc::now().timestamp()),
        api_tokens::expire_time.eq((Utc::now() + Duration::days(365)).timestamp()),
    );
    tokio::task::spawn_blocking(move || {
        diesel::insert_into(api_tokens::table)
            .values(values)
            .execute(&db_conn)
            .map(|_| ())
            .map_err(|e| HandleError::InternalError(e.to_string()))
    })
    .await
    .map_err(|_| HandleError::InternalError("Error executing request".to_string()))?
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::{
        load_db_connection_pool, models::api_tokens::APITokenData, testing as db_testing,
        DBConnectionPool,
    };

    #[tokio::test]
    async fn api_token_insert_and_retrieve() {
        // initialize db
        let pool: DBConnectionPool = load_db_connection_pool("").unwrap();
        db_testing::initialize_db_with_migration(&pool);

        // checks
        let token = APIToken::new(b"foo_bar_zen".to_vec());
        insert_token(&token, &pool).await.unwrap();
        let token_data: APITokenData = query_token(token.clone(), &pool).await.unwrap().unwrap();
        assert_eq!(token_data.token, token);
    }
}
