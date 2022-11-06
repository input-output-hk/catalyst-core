use crate::context::SharedContext;
use jortestkit::web::api_token::TokenError;
use jortestkit::web::api_token::{APIToken, APITokenManager, API_TOKEN_HEADER};
use warp::{Filter, Rejection};

pub fn filter_api_token(
    context: SharedContext,
    is_token_enabled: bool,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());

    if is_token_enabled {
        warp::header::header(API_TOKEN_HEADER)
            .and(with_context)
            .and_then(authorize_api_token)
            .and(warp::any())
            .untuple_one()
            .boxed()
    } else {
        warp::any().boxed()
    }
}

pub fn filter_admin_token(
    context: SharedContext,
    is_admin_token_enabled: bool,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    let with_context = warp::any().map(move || context.clone());
    if is_admin_token_enabled {
        warp::header::header(API_TOKEN_HEADER)
            .and(with_context)
            .and_then(authorize_admin_token)
            .and(warp::any())
            .untuple_one()
            .boxed()
    } else {
        warp::any().boxed()
    }
}

pub async fn authorize_api_token(token: String, context: SharedContext) -> Result<(), Rejection> {
    let api_token = APIToken::from_string(token).map_err(warp::reject::custom)?;

    if context.read().await.api_token().is_none() {
        return Ok(());
    }

    let manager = APITokenManager::new(context.read().await.api_token().unwrap())
        .map_err(warp::reject::custom)?;

    if !manager.is_token_valid(api_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}

pub async fn authorize_admin_token(
    admin_token: String,
    context: SharedContext,
) -> Result<(), Rejection> {
    let admin_token = APIToken::from_string(admin_token).map_err(warp::reject::custom)?;

    if context.read().await.admin_token().is_none() {
        return Ok(());
    }

    let manager = APITokenManager::new(context.read().await.admin_token().unwrap())
        .map_err(warp::reject::custom)?;

    if !manager.is_token_valid(admin_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}
