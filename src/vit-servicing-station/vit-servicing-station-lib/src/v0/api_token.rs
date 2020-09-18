use crate::db::{queries::api_tokens as api_tokens_queries, DBConnectionPool};
use crate::v0::{context::SharedContext, errors::HandleError};
use warp::{Filter, Rejection};

/// Header where token should be present in requests
pub const API_TOKEN_HEADER: &str = "API-Token";

/// API Token wrapper type
#[derive(PartialEq, Eq, Debug, Clone)]
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
        match api_tokens_queries::query_token(token, &self.connection_pool).await {
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
    base64::decode_config_buf(token.clone(), base64::URL_SAFE, &mut token_vec).map_err(|_err| {
        warp::reject::custom(HandleError::InvalidHeader(
            API_TOKEN_HEADER,
            "header should be base64 url safe decodable",
        ))
    })?;

    let api_token = APIToken(token_vec);

    match manager.is_token_valid(api_token).await {
        Ok(true) => Ok(()),
        Ok(false) => {
            crate::logging::log_rejected_api_key(token);
            Err(warp::reject::custom(HandleError::UnauthorizedToken))
        }
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
    use crate::db::{
        migrations as db_testing, models::api_tokens as api_token_model,
        models::api_tokens::APITokenData, schema::api_tokens, DBConnectionPool,
    };
    use crate::v0::api_token::{api_token_filter, APIToken, API_TOKEN_HEADER};
    use crate::v0::context::test::new_in_memmory_db_test_shared_context;
    use chrono::Utc;
    use diesel::{ExpressionMethods, RunQueryDsl};

    pub fn get_testing_token() -> (api_token_model::APITokenData, String) {
        let data = b"ffffffffffffffffffffffffffffffff".to_vec();
        let token_data = APITokenData {
            token: APIToken(data.clone()),
            creation_time: Utc::now().timestamp(),
            expire_time: Utc::now().timestamp(),
        };
        (
            token_data,
            base64::encode_config(data, base64::URL_SAFE_NO_PAD),
        )
    }

    pub fn insert_token_to_db(token: APITokenData, db: &DBConnectionPool) {
        let conn = db.get().unwrap();
        let values = (
            api_tokens::dsl::token.eq(token.token.0.clone()),
            api_tokens::dsl::creation_time.eq(token.creation_time),
            api_tokens::dsl::expire_time.eq(token.expire_time),
        );
        diesel::insert_into(api_tokens::table)
            .values(values)
            .execute(&conn)
            .unwrap();
    }

    #[tokio::test]
    async fn api_token_filter_reject() {
        let shared_context = new_in_memmory_db_test_shared_context();
        let filter = api_token_filter(shared_context).await;

        assert!(warp::test::request()
            .header(API_TOKEN_HEADER, "foobar")
            .filter(&filter)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn api_token_filter_accepted() {
        let shared_context = new_in_memmory_db_test_shared_context();

        // initialize db
        let pool = &shared_context.read().await.db_connection_pool;
        db_testing::initialize_db_with_migration(&pool.get().unwrap());
        let (token, base64_token) = get_testing_token();
        insert_token_to_db(token, pool);

        let filter = api_token_filter(shared_context.clone()).await;

        assert!(warp::test::request()
            .header(API_TOKEN_HEADER, base64_token)
            .filter(&filter)
            .await
            .is_ok());
    }
}
