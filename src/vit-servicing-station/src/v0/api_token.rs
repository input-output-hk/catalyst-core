use crate::db::DBConnectionPool;
use crate::v0::context::SharedContext;
use async_graphql::validators::InputValueValidatorExt;
use warp::filters::BoxedFilter;
use warp::{Filter, Rejection, Reply};

const API_TOKEN_HEADER: &str = "API-Token";

/// API Token wrapper type
#[derive(PartialEq, Eq)]
pub struct APIToken(String);

pub struct APITokenManager {
    connection_pool: DBConnectionPool,
}

impl APITokenManager {
    fn new(connection_pool: DBConnectionPool) -> Self {
        Self { connection_pool }
    }

    async fn is_token_valid(&self, token: APIToken) -> bool {
        false
    }

    async fn revoke_token(&self, token: APIToken) -> Result<(), ()> {
        Ok(())
    }
}

async fn reject(token: String, context: SharedContext) -> Result<(), Rejection> {
    let manager = APITokenManager::new(context.read().await.db_connection_pool.clone());
    if manager.is_token_valid(APIToken(token)).await {
        return Ok(());
    }
    Err(warp::reject())
}

pub async fn api_token_filter(
    context: SharedContext,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());
    warp::header::header(API_TOKEN_HEADER)
        .and(with_context)
        .and_then(reject)
        .and(warp::any())
        .untuple_one()
}
