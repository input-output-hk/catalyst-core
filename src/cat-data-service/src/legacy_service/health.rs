use crate::{
    service::Error,
    settings::{RetryAfterParams, RETRY_AFTER_HTTP_DATE_DEFAULT},
    state::State,
};
use axum::{
    body,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, put},
    Router,
};
use std::sync::Arc;

use super::handle_result;

pub fn health(state: Arc<State>) -> Router {
    Router::new()
        .route(
            "/health/ready",
            get({
                let state = state.clone();
                move || async { handle_result(ready_exec(state).await) }
            }),
        )
        .route(
            "/health/live",
            get(|| async { handle_result(live_exec().await) }),
        )
        .route(
            "/health/retry-after",
            put({
                let state = state.clone();
                move |params| async { retry_after_exec(params, state).await }
            }),
        )
}

async fn ready_exec(state: Arc<State>) -> Result<bool, Error> {
    tracing::debug!("health ready exec");

    state.event_db.schema_version_check().await?;
    Ok(true)
}

async fn live_exec() -> Result<bool, Error> {
    tracing::debug!("health live exec");

    Ok(true)
}

async fn retry_after_exec(Query(params): Query<RetryAfterParams>, state: Arc<State>) -> Response {
    tracing::debug!(params = format!("{params:?}"), "health retry_after exec");
    println!("query params: {params:?}");

    match params {
        // Request without query parameters resets env vars.
        // Sets `RETRY_AFTER_HTTP_DATE` to the default value.
        // Sets `RETRY_AFTER_DELAY_SECONDS` to initial state value.
        RetryAfterParams {
            http_date: None,
            delay_seconds: None,
        } => {
            tracing::debug!("RETRY_AFTER parameters RESET");

            let date_str = RETRY_AFTER_HTTP_DATE_DEFAULT;
            RetryAfterParams::http_date_set_var(date_str);

            let delay_secs = state.delay_seconds;
            RetryAfterParams::delay_seconds_set_var(delay_secs);
        }
        // Request with `http_date` query parameter sets `RETRY_AFTER_HTTP_DATE` env var,
        // and resets `RETRY_AFTER_DELAY_SECONDS` to initial state value.
        RetryAfterParams {
            http_date: Some(http_date),
            delay_seconds: _,
        } => {
            let date_str = http_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            RetryAfterParams::http_date_set_var(&date_str);

            let delay_secs = state.delay_seconds;
            RetryAfterParams::delay_seconds_set_var(delay_secs);
        }
        // Request with `delay_seconds` query parameter sets `RETRY_AFTER_DELAY_SECONDS` env var,
        // and resets `RETRY_AFTER_HTTP_DATE` to the default value.
        RetryAfterParams {
            http_date: None,
            delay_seconds: Some(delay_seconds),
        } => {
            let date_str = RETRY_AFTER_HTTP_DATE_DEFAULT;
            RetryAfterParams::http_date_set_var(date_str);

            RetryAfterParams::delay_seconds_set_var(delay_seconds);
        }
    }

    (StatusCode::NO_CONTENT, body::Empty::new()).into_response()
}

/// Need to setup and run a test event db instance
/// To do it you can use the following commands:
/// Prepare docker images
/// ```
/// earthly ./containers/event-db-migrations+docker --data=test
/// ```
/// Run event-db container
/// ```
/// docker-compose -f src/event-db/docker-compose.yml up migrations
/// ```
/// Also need establish `EVENT_DB_URL` env variable with the following value
/// ```
/// EVENT_DB_URL="postgres://catalyst-event-dev:CHANGE_ME@localhost/CatalystEventDev"
/// ```
/// https://github.com/input-output-hk/catalyst-core/tree/main/src/event-db/Readme.md
#[cfg(test)]
mod tests {
    use crate::{
        legacy_service::app,
        settings::{RETRY_AFTER_HTTP_DATE_DEFAULT, RETRY_AFTER_HTTP_DATE_ENVVAR},
        state::State,
    };
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_ready_test() {
        let state = Arc::new(State::new(None, None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri("/health/ready".to_string())
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
                .as_str(),
            r#"true"#
        );
    }

    #[tokio::test]
    async fn health_live_test() {
        let state = Arc::new(State::new(None, None).await.unwrap());
        let app = app(state);

        let request = Request::builder()
            .uri("/health/live".to_string())
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            String::from_utf8(response.into_body().data().await.unwrap().unwrap().to_vec())
                .unwrap()
                .as_str(),
            r#"true"#
        );
    }

    #[tokio::test]
    async fn health_retry_after_test() {
        use crate::settings::RETRY_AFTER_DELAY_SECONDS_ENVVAR;
        const INITIAL_DELAY_SECONDS: u64 = 200;

        let state = Arc::new(State::new(None, Some(INITIAL_DELAY_SECONDS)).await.unwrap());
        let app = app(state);

        // Request without query parameters resets env vars.
        // Sets `RETRY_AFTER_HTTP_DATE` to the default value.
        // Sets `RETRY_AFTER_DELAY_SECONDS` to initial state value.
        let request = Request::builder()
            .uri("/health/retry-after".to_string())
            .method("PUT")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let delay_seconds_envvar_value = std::env::var(RETRY_AFTER_DELAY_SECONDS_ENVVAR).unwrap();
        assert_eq!(
            delay_seconds_envvar_value,
            INITIAL_DELAY_SECONDS.to_string()
        );

        let http_date_envvar_value = std::env::var(RETRY_AFTER_HTTP_DATE_ENVVAR).unwrap();
        assert_eq!(http_date_envvar_value, RETRY_AFTER_HTTP_DATE_DEFAULT);

        // Request with `http_date` query parameter sets `RETRY_AFTER_HTTP_DATE` env var,
        // and resets `RETRY_AFTER_DELAY_SECONDS` to initial state value.
        let request = Request::builder()
            .uri("/health/retry-after?http-date=2040-01-01T00:00:00Z".to_string())
            .method("PUT")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let http_date_envvar_value = std::env::var(RETRY_AFTER_HTTP_DATE_ENVVAR).unwrap();
        assert_eq!(http_date_envvar_value, "2040-01-01T00:00:00Z");

        let delay_seconds_envvar_value = std::env::var(RETRY_AFTER_DELAY_SECONDS_ENVVAR).unwrap();
        assert_eq!(
            delay_seconds_envvar_value,
            INITIAL_DELAY_SECONDS.to_string()
        );

        // Request with `delay_seconds` query parameter sets `RETRY_AFTER_DELAY_SECONDS` env var,
        // and resets `RETRY_AFTER_HTTP_DATE` to the default value.
        let request = Request::builder()
            .uri("/health/retry-after?delay-seconds=900".to_string())
            .method("PUT")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        let http_date_envvar_value = std::env::var(RETRY_AFTER_HTTP_DATE_ENVVAR).unwrap();
        assert_eq!(http_date_envvar_value, RETRY_AFTER_HTTP_DATE_DEFAULT);

        let delay_seconds_envvar_value = std::env::var(RETRY_AFTER_DELAY_SECONDS_ENVVAR).unwrap();
        assert_eq!(delay_seconds_envvar_value, "900".to_string());
    }
}
