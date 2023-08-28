//! Poem Service for cat-data-service endpoints.

use super::api::api;
use crate::service::docs::docs;
use crate::service::utilities::metrics_tracing::{init_prometheus, log_requests};
use crate::service::Error;
use poem::middleware::Cors;
use poem::Endpoint;
use poem::{endpoint::PrometheusExporter, listener::TcpListener, EndpointExt, Route};
use std::net::SocketAddr;

pub fn cors_layer() -> Cors {
    Cors::new()
}

pub fn app(addr: &SocketAddr) -> impl EndpointExt {
    let api = api(addr);
    let docs = docs(&api);

    Route::new()
        .nest("/api", api)
        .nest("/docs", docs)
        .around(|endp, req| async move { Ok(log_requests(endp, req).await) })
}

pub fn metrics_app() -> impl Endpoint {
    let prometheus_controller = init_prometheus();

    Route::new().nest(
        "/metrics",
        PrometheusExporter::with_controller(prometheus_controller),
    )
}

pub async fn run_service<E: Endpoint + 'static>(
    app: E,
    addr: &SocketAddr,
    name: &str,
) -> Result<(), Error> {
    tracing::info!("Starting {name}...");
    tracing::info!("Listening on {addr}");

    poem::Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .map_err(Error::Io)
}
