//! Poem Service for cat-data-service endpoints.

use crate::service::ui::stoplight_elements;
use crate::service::Error;

use opentelemetry::sdk::{
    export::metrics::aggregation,
    metrics::{
        controllers::{self, BasicController},
        processors, selectors,
    },
};
use poem::middleware::Cors;
use poem::{
    endpoint::PrometheusExporter, listener::TcpListener, middleware::OpenTelemetryMetrics,
    EndpointExt, Route,
};
use poem_openapi::{param::Query, payload::PlainText, OpenApi, OpenApiService};
use std::net::SocketAddr;

struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
}

fn init_prometheus() -> BasicController {
    controllers::basic(processors::factory(
        selectors::simple::histogram([
            1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 70.0, 100.0, 200.0, 300.0, 400.0, 500.0, 700.0,
            1000.0, 1500.0, 3000.0,
        ]),
        aggregation::cumulative_temporality_selector(),
    ))
    .build()
}

pub async fn run_service(
    addr: &SocketAddr,
    metrics_addr: &Option<SocketAddr>,
) -> Result<(), Error> {
    tracing::info!("Starting Poem Service ...");
    tracing::info!("Listening on {addr}");
    tracing::info!("Metrics on {metrics_addr:?}");

    let server_host = format!("http://{}:{}/api", addr.ip(), addr.port());

    let api_service = OpenApiService::new(Api, "Hello World", "1.0").server(server_host);

    let spec = api_service.spec();

    let swagger_ui = api_service.swagger_ui();
    let rapidoc_ui = api_service.rapidoc();
    let redoc_ui = api_service.redoc();
    let openapi_explorer = api_service.openapi_explorer();
    let stoplight_ui = stoplight_elements::create_endpoint(&spec);

    let prometheus_controller = init_prometheus();

    let app = Route::new()
        .nest("/api", api_service)
        .nest("/docs/", stoplight_ui)
        .nest("/docs/swagger_ui", swagger_ui)
        .nest("/docs/redoc", redoc_ui)
        .nest("/docs/rapidoc", rapidoc_ui)
        .nest("/docs/openapi_explorer", openapi_explorer)
        .at(
            "/docs/cat-data-service.json",
            poem::endpoint::make_sync(move |_| spec.clone()),
        )
        .nest(
            "/prometheus_metrics",
            PrometheusExporter::with_controller(prometheus_controller),
        )
        .with(Cors::new())
        .with(OpenTelemetryMetrics::new());

    poem::Server::new(TcpListener::bind(addr))
        .run(app)
        .await
        .map_err(Error::IoError)
}
