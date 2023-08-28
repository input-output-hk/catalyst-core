//! Poem Service for cat-data-service endpoints.

use crate::service::docs::docs;
use crate::service::Error;

use crate::service::utilities::metrics_tracing::{init_prometheus, log_requests};
use crate::service::api::mk_api;

use poem::middleware::Cors;
use poem::{
    endpoint::PrometheusExporter, listener::TcpListener, middleware::OpenTelemetryMetrics,
    EndpointExt, Route,
};
use std::net::SocketAddr;


pub async fn run_service(
    addr: &SocketAddr,
    metrics_addr: &Option<SocketAddr>,
) -> Result<(), Error> {
    tracing::info!("Starting Poem Service ...");
    tracing::info!("Listening on {addr}");
    tracing::info!("Metrics on {metrics_addr:?}");

    let api = mk_api(&addr);
    let docs = docs();

    let prometheus_controller = init_prometheus();

    let app = Route::new()
        .nest("/api", api)
        .nest("/docs/",docs)
        .nest(
            "/metrics",
            PrometheusExporter::with_controller(prometheus_controller),
        )
        .with(Cors::new())
        .with(OpenTelemetryMetrics::new())
        .around(|ep, req| async move {
            Ok(log_requests(ep, req).await)
        });

    poem::Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .map_err(Error::IoError)
}

