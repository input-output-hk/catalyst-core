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
    let api_root = warp::path!("api" / ..);

    let v0_root = warp::path!("v0" / ..);
    let service_version_root = warp::path!("vit-version" / ..);
    // log request statistics
    let log = warp::log::custom(|info| {
        tracing::info!("Request elapsed time: {}ms", info.elapsed().as_millis())
    });
    let v0 = endpoints::filter(v0_root.boxed(), ctx.clone(), enable_api_tokens).await;

    let service_version =
        endpoints::service_version::filter(service_version_root.boxed(), ctx).await;

    api_root
        .and(v0.or(service_version))
        .with(warp::trace::named(V0_REQUEST_TRACE_NAME))
        .recover(errors::handle_rejection)
        .with(log)
}
