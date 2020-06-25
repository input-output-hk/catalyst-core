use crate::db::{
    models::api_tokens as api_token_model,
    schema::{api_tokens, api_tokens::dsl::api_tokens as api_tokens_dsl},
    DBConnectionPool,
};
use crate::v0::api_token::APIToken;
use crate::v0::errors::HandleError;
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
