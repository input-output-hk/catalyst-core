//! Metrics and Tracing functionality for the API
use opentelemetry::sdk::{
    export::metrics::aggregation,
    metrics::{
        controllers::{self, BasicController},
        processors, selectors,
    },
};
use poem::Route;
use poem::{
    middleware::{CorsEndpoint, OpenTelemetryMetricsEndpoint},
    Endpoint, Request, Response,
};
use poem_openapi::OperationId;
use std::time::Instant;
use std::{env, sync::Arc};
use tracing::{info, span};

use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

/// Get an anonymized client ID from the request.
///
/// This simply takes the clients IP address,
/// adds a supplied key to it, and hashes the result.
///
/// The Hash is unique per client IP, but not able to
/// be reversed or analysed without both the client IP and the key.
fn anonymous_client_id(req: &Request) -> String {
    // Get the Anonymous Client ID Key.
    // In production this is a secret key.
    // In development this is the default value specified here.
    let client_anonymous_key = env::var("CLIENT_ID_KEY")
        .unwrap_or_else(|_| String::from("3db5301e-40f2-47ed-ab11-55b37674631a"));

    let mut b2b = Blake2b::new(16); // We are going to represent it as a UUID.
    let mut out = [0; 16];

    b2b.input_str(&client_anonymous_key);
    b2b.input_str(&req.remote_addr().to_string());
    b2b.result(&mut out);

    // Note: This will only panic if the `out` is not 16 bytes long.
    // Which it is.
    // Therefore the `unwrap()` is safe and will not cause a panic here under any circumstances.
    uuid::Builder::from_slice(&out)
        .unwrap()
        .into_uuid()
        .hyphenated()
        .to_string()
}

/// Log all requests, with important tracing data.
///
/// ## Arguments
/// * `ep` - Endpoint of the request being made.
/// * `req` - Request being made
///
pub async fn log_requests(
    ep: Arc<OpenTelemetryMetricsEndpoint<CorsEndpoint<Route>>>,
    req: Request,
) -> Response {
    let uri = req.uri().clone();

    let client_id = anonymous_client_id(&req); // Get the clients anonymous unique id.
    let conn_id = uuid::Uuid::new_v4().to_string(); // Make a random V4 UUID to track the connection.

    let _span = span!(tracing::Level::INFO, "request", conn_id, client_id);

    // TODO use tokio metrics to get better async stats.  Discuss with team first.
    let start = Instant::now();

    info!(
        phase = "request",
        ver = ?req.version(),
        path = uri.path(),
        query = uri.query().unwrap_or(""),
        method = req.method().as_str(),
        content = req.content_type().unwrap_or("Undefined"),
        // size = req.content_length(),  # Nice to know how big a request was.
    );

    let resp = ep.get_response(req).await;

    // Wall Time taken to execute function, as ms to 3 decimal places.
    // We are OK with the change in precision for this.
    #[allow(clippy::cast_precision_loss)]
    let elapsed_ms = start.elapsed().as_nanos() as f64 / 1_000.0;

    // The OpenAPI Operation ID of this request.
    let oid = match resp.data::<OperationId>() {
        Some(oid) => oid.to_string(),
        None => "None".to_string(),
    };

    info!(
        phase = "response",
        status = resp.status().as_str(),
        reason = resp.status().canonical_reason(),
        oid,
        elapsed_ms,
        content = resp.content_type().unwrap_or("Undefined"),
        // size = req.content_length(),  # Nice to know how big a response was.
    );

    resp
}

/// Initialize Prometheus metrics.
///
/// ## Returns
///
/// Returns a prometheus controller configured to collect metrics for the service.
#[must_use]
pub fn init_prometheus() -> BasicController {
    controllers::basic(processors::factory(
        selectors::simple::histogram([
            1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 70.0, 100.0, 200.0, 300.0, 400.0, 500.0, 700.0,
            1000.0, 1500.0, 3000.0,
        ]),
        aggregation::cumulative_temporality_selector(),
    ))
    .build()
}
