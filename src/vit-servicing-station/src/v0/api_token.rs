use crate::db::{
    models::api_token,
    schema::{api_tokens, api_tokens::dsl::api_tokens as api_tokens_dsl},
    DBConnectionPool,
};
use crate::v0::{context::SharedContext, errors::HandleError};
use diesel::query_dsl::RunQueryDsl;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use warp::{Filter, Rejection};

/// Header where token should be present in requests
const API_TOKEN_HEADER: &str = "API-Token";

/// API Token wrapper type
#[derive(PartialEq, Eq)]
pub struct APIToken(Vec<u8>);

/// API token manager is an abstraction on the API tokens for the service
/// The main idea is to keep the service agnostic of what kind of backend we are using such task.
/// Right now we rely on a SQLlite connection. But in the future it maybe be something else like a
/// REDIS, or some other hybrid system.
pub struct APITokenManager {
    connection_pool: DBConnectionPool,
}

impl From<&[u8]> for APIToken {
    fn from(data: &[u8]) -> Self {
        Self(data.to_vec())
    }
}

impl AsRef<[u8]> for APIToken {
    fn as_ref(&self) -> &[u8] {
        self.0.as_slice()
    }
}

impl APIToken {
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }
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
                .filter(api_tokens::token.eq(token.as_ref()))
                .first::<api_token::APITokenData>(&db_conn)
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

    let mut token_vec: Vec<u8> = Vec::new();
    base64::decode_config_buf(token, base64::URL_SAFE, &mut token_vec).map_err(|_err| {
        warp::reject::custom(HandleError::InvalidHeader(
            API_TOKEN_HEADER,
            "header should be base64 url safe decodable",
        ))
    })?;

    let api_token = APIToken(token_vec);

    match manager.is_token_valid(api_token).await {
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

#[cfg(test)]
mod test {
    use crate::v0::api_token::{api_token_filter, API_TOKEN_HEADER};
    use crate::v0::context::test::new_test_shared_context;
    use crate::v0::context::SharedContext;
    use crate::v0::context::{new_shared_context, test::new_empty_test_shared_context};
    use chrono::DateTime;

    #[tokio::test]
    async fn api_token_filter_reject() {
        let shared_context = new_empty_test_shared_context();
        let filter = api_token_filter(shared_context).await;

        assert!(warp::test::request()
            .header(API_TOKEN_HEADER, "foobar")
            .filter(&filter)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn api_token_filter_accepted() {
        let shared_context = new_test_shared_context(
            "./db/tests/vit_station_test.db",
            "./resources/tests/block0.bin",
        );
        let filter = api_token_filter(shared_context).await;
        let base64_token = "ZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmZmY=";
        let token = assert!(warp::test::request()
            .header(API_TOKEN_HEADER, base64_token)
            .filter(&filter)
            .await
            .is_ok());
    }
}
