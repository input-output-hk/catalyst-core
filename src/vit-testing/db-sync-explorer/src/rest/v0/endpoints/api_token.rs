use crate::rest::v0::context::SharedContext;
use jortestkit::web::api_token::{APIToken, APITokenManager, TokenError, API_TOKEN_HEADER};
use warp::{Filter, Rejection};

pub async fn api_token_filter(
    context: SharedContext,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::header(API_TOKEN_HEADER)
        .and(warp::any().map(move || context.clone()))
        .and_then(authorize_token)
        .and(warp::any())
        .untuple_one()
}

pub async fn authorize_token(token: String, context: SharedContext) -> Result<(), Rejection> {
    let api_token = APIToken::from_string(token).map_err(warp::reject::custom)?;

    let context = context.read().await;
    let maybe_token = context.token().as_ref();

    if maybe_token.is_none() {
        return Ok(());
    }

    let manager =
        APITokenManager::new(maybe_token.unwrap().clone()).map_err(warp::reject::custom)?;

    if !manager.is_token_valid(api_token) {
        return Err(warp::reject::custom(TokenError::UnauthorizedToken));
    }
    Ok(())
}
