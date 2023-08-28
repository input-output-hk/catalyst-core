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
    Endpoint, Request,
};
use poem_openapi::OperationId;
use std::sync::Arc;

/// Log all requests, with important tracing data.
pub async fn log_requests(
    ep: Arc<OpenTelemetryMetricsEndpoint<CorsEndpoint<Route>>>,
    req: Request,
) {
    let uri = req.uri().clone();
    let resp = ep.get_response(req).await;

    if let Some(operation_id) = resp.data::<OperationId>() {
        println!("[{}]{} {}", operation_id, uri, resp.status());
    } else {
        println!("{} {}", uri, resp.status());
    }
}

/// Initialize Prometheus metrics.
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
