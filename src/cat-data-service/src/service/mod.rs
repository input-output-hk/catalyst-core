use crate::state::State;
use axum::{
    extract::MatchedPath,
    http::{Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use serde::Serialize;
use std::{future::ready, net::SocketAddr, sync::Arc, time::Instant};
use tower_http::cors::{Any, CorsLayer};

mod health;
mod v0;
mod v1;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Cannot run service, error: {0}")]
    CannotRunService(String),
    #[error(transparent)]
    EventDbError(#[from] event_db::error::Error),
}

#[derive(Serialize, Debug)]
pub struct ErrorMessage {
    error: String,
}

pub fn app(state: Arc<State>) -> Router {
    // build our application with a route
    let v0 = v0::v0(state.clone());
    let v1 = v1::v1(state);
    let health = health::health();
    Router::new().nest("/api", v1.merge(v0)).merge(health)
}

fn metrics_app() -> Router {
    let recorder_handle = setup_metrics_recorder();
    Router::new().route("/metrics", get(move || ready(recorder_handle.render())))
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(Any)
}

async fn run_service(app: Router, addr: &SocketAddr, name: &str) -> Result<(), Error> {
    tracing::info!("Starting {name}...");
    tracing::info!("Listening on {addr}");

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Error::CannotRunService(e.to_string()))?;
    Ok(())
}

pub async fn run(
    service_addr: &SocketAddr,
    metrics_addr: &Option<SocketAddr>,
    state: Arc<State>,
) -> Result<(), Error> {
    let cors = cors_layer();
    if let Some(metrics_addr) = metrics_addr {
        let service_app = app(state)
            .layer(cors.clone())
            .route_layer(middleware::from_fn(track_metrics));
        let metrics_app = metrics_app().layer(cors);

        tokio::try_join!(
            run_service(service_app, service_addr, "service"),
            run_service(metrics_app, metrics_addr, "metrics"),
        )?;
    } else {
        let service_app = app(state).layer(cors);

        run_service(service_app, service_addr, "service").await?;
    }
    Ok(())
}

fn handle_result<T: Serialize>(res: Result<T, Error>) -> Response {
    match res {
        Ok(res) => (StatusCode::OK, Json(res)).into_response(),
        Err(Error::EventDbError(event_db::error::Error::NotFound(error))) => {
            (StatusCode::NOT_FOUND, Json(ErrorMessage { error })).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorMessage {
                error: error.to_string(),
            }),
        )
            .into_response(),
    }
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

async fn track_metrics<T>(req: Request<T>, next: Next<T>) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::increment_counter!("http_requests_total", &labels);
    metrics::histogram!("http_requests_duration_seconds", latency, &labels);

    response
}

#[cfg(test)]
pub mod tests {
    use std::str::FromStr;

    pub fn body_data_json_check(body_data: Vec<u8>, expected_json: serde_json::Value) -> bool {
        serde_json::Value::from_str(String::from_utf8(body_data).unwrap().as_str()).unwrap()
            == expected_json
    }
}
