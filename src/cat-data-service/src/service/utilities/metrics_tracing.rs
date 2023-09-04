//! Metrics and Tracing functionality for the API
use poem::{Endpoint, Request, Response};
use poem_openapi::OperationId;
use prometheus::Registry;
use std::time::Instant;
use tracing::{info, info_span, Instrument};

use cryptoxide::blake2b::Blake2b;
use cryptoxide::digest::Digest;

use crate::settings::CLIENT_ID_KEY;

/// Get an anonymized client ID from the request.
///
/// This simply takes the clients IP address,
/// adds a supplied key to it, and hashes the result.
///
/// The Hash is unique per client IP, but not able to
/// be reversed or analysed without both the client IP and the key.
fn anonymous_client_id(req: &Request) -> String {
    let mut b2b = Blake2b::new(16); // We are going to represent it as a UUID.
    let mut out = [0; 16];

    b2b.input_str(CLIENT_ID_KEY.as_str());
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

/// This is where the request actually gets processed and logs.
/// We do this because spans do not play nicely with async code.
/// This allows us to properly instrument the span to the request.
async fn process_requests<E: Endpoint>(ep: E, req: Request) -> Response {
    let uri = req.uri().clone();

    info!(
        phase = "request",
        ver = ?req.version(),
        path = uri.path(),
        query = uri.query().unwrap_or(""),
        method = req.method().as_str(),
        content = req.content_type().unwrap_or("Undefined"),
        // size = req.content_length(),  # Nice to know how big a request was.
    );

    // Get absolute start of processing the request.
    let start = Instant::now();

    let resp = ep.get_response(req).await;

    // Wall Time taken to execute function, as ms to 3 decimal places.
    // We are OK with the change in precision for this.
    #[allow(clippy::cast_precision_loss)]
    let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;

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

/// Log all requests, with important tracing data.
///
/// ## Arguments
/// * `ep` - Endpoint of the request being made.
/// * `req` - Request being made
///
pub async fn log_requests<E: Endpoint>(ep: E, req: Request) -> Response {
    let client_id = anonymous_client_id(&req); // Get the clients anonymous unique id.
    let conn_id = uuid::Uuid::new_v4().to_string(); // Make a random V4 UUID to track the connection.

    process_requests(ep, req)
        .instrument(
            info_span!(target: "api_request", "request", conn_id = %conn_id, client_id = %client_id),
        )
        .await
}

/// Initialize Prometheus metrics.
///
/// ## Returns
///
/// Returns a prometheus controller configured to collect metrics for the service.
#[must_use]
pub fn init_prometheus() -> Registry {
    Registry::default()
}
