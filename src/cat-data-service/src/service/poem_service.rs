//! Poem Service for cat-data-service endpoints.
//!
//! This provides only the primary entrypoint to the service.

use crate::service::docs::docs;
use crate::service::Error;

use crate::service::api::mk_api;
use crate::service::utilities::metrics_tracing::{init_prometheus, log_requests};

use poem::middleware::{Cors, OpenTelemetryMetrics};
use poem::{endpoint::PrometheusExporter, listener::TcpListener, EndpointExt, Route};
use std::net::SocketAddr;

/// Run the Poem Service
///
/// This provides only the primary entrypoint to the service.
/// addr: &SocketAddr - the address to listen on
///
pub async fn run_service(addr: &SocketAddr) -> Result<(), Error> {
    tracing::info!("Starting Poem Service ...");
    tracing::info!("Listening on {addr}");

    let api_service = mk_api(addr);
    let docs = docs(&api_service);

    let prometheus_controller = init_prometheus();

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs", docs)
        .nest(
            "/metrics",
            PrometheusExporter::with_controller(prometheus_controller),
        )
        .with(Cors::new())
        .with(OpenTelemetryMetrics::new());
    //.around(|ep, req| async move { Ok(log_requests(ep, req).await) });

    poem::Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .map_err(Error::Io)
}
