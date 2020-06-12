use crate::db::{
    models::api_token,
    schema::{api_tokens, api_tokens::dsl::api_tokens as api_tokens_dsl},
    DBConnectionPool,
};
use crate::v0::{context::SharedContext, errors::HandleError};
use diesel::query_dsl::{filter_dsl::FilterDsl, RunQueryDsl};
use diesel::{ExpressionMethods, OptionalExtension};
use warp::{Filter, Rejection};

/// Header where token should be present in requests
const API_TOKEN_HEADER: &str = "API-Token";

/// API Token wrapper type
#[derive(PartialEq, Eq)]
pub struct APIToken(String);

/// API token manager is an abstraction on the API tokens for the service
/// The main idea is to keep the service agnostic of what kind of backend we are using such task.
/// Right now we rely on a SQLlite connection. But in the future it maybe be something else like a
/// REDIS, or some other hybrid system.
pub struct APITokenManager {
    connection_pool: DBConnectionPool,
}

impl APITokenManager {
    fn new(connection_pool: DBConnectionPool) -> Self {
        Self { connection_pool }
    }

    async fn is_token_valid(&self, token: APIToken) -> Result<bool, HandleError> {
        let db_conn = self
            .connection_pool
            .get()
            .map_err(HandleError::DatabaseError)?;

        match tokio::task::spawn_blocking(move || {
            api_tokens_dsl
                .filter(api_tokens::token.eq(token.0))
                .first::<api_token::APIToken>(&db_conn)
                .optional()
        })
        .await
        .map_err(|_| HandleError::InternalError("Error executing request".to_string()))?
        {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(HandleError::InternalError(format!(
                "Error retrieving token: {}",
                e
            ))),
        }
    }

    #[allow(dead_code)]
    async fn revoke_token(&self, _token: APIToken) -> Result<(), ()> {
        Ok(())
    }
}

async fn authorize_token(token: String, context: SharedContext) -> Result<(), Rejection> {
    let manager = APITokenManager::new(context.read().await.db_connection_pool.clone());
    match manager.is_token_valid(APIToken(token)).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(warp::reject::custom(HandleError::UnauthorizedToken)),
        Err(e) => Err(warp::reject::custom(e)),
    }
}

/// A warp filter that checks authorization through API tokens.
/// The header `API_TOKEN_HEADER` should be present and valid otherwise the request is rejected.
pub async fn api_token_filter(
    context: SharedContext,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());
    warp::header::header(API_TOKEN_HEADER)
        .and(with_context)
        .and_then(authorize_token)
        .and(warp::any())
        .untuple_one()
}
