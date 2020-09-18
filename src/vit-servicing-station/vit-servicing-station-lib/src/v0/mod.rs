pub mod api_token;
pub mod context;
pub mod endpoints;
pub mod errors;
pub mod result;

use warp::{Filter, Rejection, Reply};

pub async fn filter(
    ctx: context::SharedContext,
    enable_api_tokens: bool,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let root = warp::path!("api" / "v0" / ..);

    // log request statistics
    let log = warp::log::custom(|info| crate::logging::log_request_elapsed_time(info.elapsed()));

    endpoints::filter(root.boxed(), ctx, enable_api_tokens)
        .await
        .recover(errors::handle_rejection)
        .with(log)
}
