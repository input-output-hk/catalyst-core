pub mod api_token;
pub mod context;
pub mod endpoints;
pub mod errors;
pub mod result;

use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};

pub async fn filter(
    ctx: context::SharedContext,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let root = warp::path!("api" / "v0" / ..).and(api_token::api_token_filter(ctx.clone()).await);

    endpoints::filter(root.boxed(), ctx)
        .await
        .recover(errors::handle_rejection)
}
