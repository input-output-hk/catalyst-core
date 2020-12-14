pub mod api_token;
pub mod context;
pub mod endpoints;
pub mod errors;
pub mod result;
use warp::{Filter, Rejection, Reply};

const V0_REQUEST_TRACE_NAME: &str = "v0_request";

pub async fn filter(
    ctx: context::SharedContext,
    enable_api_tokens: bool,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let root = warp::path!("api" / "v0" / ..);

    // log request statistics
    let log = warp::log::custom(|info| {
        tracing::info!("Request elapsed time: {}ms", info.elapsed().as_millis())
    });

    endpoints::filter(root.boxed(), ctx, enable_api_tokens)
        .await
        .with(warp::trace::named(V0_REQUEST_TRACE_NAME))
        .recover(errors::handle_rejection)
        .with(log)
}
