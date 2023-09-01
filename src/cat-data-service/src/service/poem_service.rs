//! Poem Service for cat-data-service endpoints.
//!
//! This provides only the primary entrypoint to the service.


use crate::service::docs::docs;
use crate::service::Error;

use crate::service::api::mk_api;
use crate::service::utilities::metrics_tracing::{init_prometheus, log_requests};
use crate::service::utilities::catch_panic::{ServicePanicHandler, set_panic_hook};
use crate::settings::API_URL_PREFIX;

use poem::endpoint::PrometheusExporter;
use poem::listener::TcpListener;
use poem::middleware::{CatchPanic, Cors, OpenTelemetryMetrics};
use poem::{EndpointExt, Route};
use std::net::SocketAddr;

/// Run the Poem Service
///
/// This provides only the primary entrypoint to the service.
///
/// # Arguments
///
/// *`addr`: &`SocketAddr` - the address to listen on
///
/// # Errors
///
/// * `Error::CannotRunService` - cannot run the service
/// * `Error::EventDbError` - cannot connect to the event db
/// * `Error::IoError` - An IO error has occurred.
///
pub async fn run_service(addr: &SocketAddr) -> Result<(), Error> {
    tracing::info!("Starting Poem Service ...");
    tracing::info!("Listening on {addr}");

    // Set a custom panic hook, so we can catch panics and not crash the service.
    // And also get data from the panic so we can log it.
    // Panics will cause a 500 to be sent with minimal information we can use to
    // help find them in the logs if they happen in production.
    set_panic_hook();

    let api_service = mk_api(addr);
    let docs = docs(&api_service);

    let prometheus_controller = init_prometheus();

    let app = Route::new()
        .nest(API_URL_PREFIX.as_str(), api_service)
        .nest("/docs", docs)
        .nest(
            "/metrics",
            PrometheusExporter::with_controller(prometheus_controller),
        )
        .with(Cors::new())
        .with(OpenTelemetryMetrics::new())
        .with(CatchPanic::new().with_handler(ServicePanicHandler))
        .around(|ep, req| async move { Ok(log_requests(ep, req).await) });

    poem::Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .map_err(Error::Io)
}
