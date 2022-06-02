pub mod api_token;
pub mod context;
pub mod endpoints;
pub mod errors;
pub mod result;

use warp::{Filter, Rejection, Reply};

const V0_REQUEST_TRACE_NAME: &str = "v0_request";

pub async fn filter(
    ctx: context::SharedContext,
    snapshot_rx: snapshot_service::SharedContext,
    snapshot_tx: snapshot_service::UpdateHandle,
    enable_api_tokens: bool,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let api_root = warp::path!("api" / ..);

    let v0_root = warp::path!("v0" / ..);
    let service_version_root = warp::path!("vit-version" / ..);
    // log request statistics
    let log = warp::filters::trace::trace(|info| {
        use http_zipkin::get_trace_context;
        use tracing::field::Empty;
        let span = tracing::span!(
            tracing::Level::DEBUG,
            "rest_api_request",
            method = %info.method(),
            path = info.path(),
            version = ?info.version(),
            remote_addr = Empty,
            trace_id = Empty,
            span_id = Empty,
            parent_span_id = Empty,
        );
        if let Some(remote_addr) = info.remote_addr() {
            span.record("remote_addr", &remote_addr.to_string().as_str());
        }
        if let Some(trace_context) = get_trace_context(info.request_headers()) {
            span.record("trace_id", &trace_context.trace_id().to_string().as_str());
            span.record("span_id", &trace_context.span_id().to_string().as_str());
            if let Some(parent_span_id) = trace_context.parent_id() {
                span.record("parent_span_id", &parent_span_id.to_string().as_str());
            }
        }
        span
    });

    let v0 = endpoints::filter(
        v0_root.boxed(),
        ctx.clone(),
        snapshot_rx,
        snapshot_tx,
        enable_api_tokens,
    )
    .await;

    let service_version =
        endpoints::service_version::filter(service_version_root.boxed(), ctx).await;

    api_root
        .and(v0.or(service_version))
        .with(warp::trace::named(V0_REQUEST_TRACE_NAME))
        .recover(errors::handle_rejection)
        .with(log)
}
