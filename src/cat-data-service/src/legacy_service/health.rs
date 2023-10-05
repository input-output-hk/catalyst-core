use crate::{
    service::Error,
    settings::{
        RetryAfterParams, RETRY_AFTER_DELAY_SECONDS_ENVVAR, RETRY_AFTER_HTTP_DATE_DEFAULT,
        RETRY_AFTER_HTTP_DATE_ENVVAR,
    },
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

async fn retry_after_exec(params: Query<RetryAfterParams>, state: Arc<State>) -> Response {
    tracing::debug!(params = format!("{params:?}"), "health retry_after exec");
    match params.0 {
        RetryAfterParams {
            http_date: None,
            delay_seconds: None,
        } => {
            tracing::debug!("RETRY_AFTER RESET");
            let date_str = RETRY_AFTER_HTTP_DATE_DEFAULT;
            tracing::debug!(http_date = date_str, "HTTP_DATE RESET");
            std::env::set_var(RETRY_AFTER_HTTP_DATE_ENVVAR, date_str);

            let delay_secs = state.delay_seconds.to_string();
            tracing::debug!(delay_seconds = delay_secs, "DELAY_SECONDS RESET");
            std::env::set_var(RETRY_AFTER_DELAY_SECONDS_ENVVAR, delay_secs);
        }
        RetryAfterParams {
            http_date: Some(http_date),
            delay_seconds: _,
        } => {
            let date_str = http_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            tracing::debug!(http_date = date_str, "HTTP_DATE SET");
            std::env::set_var(RETRY_AFTER_HTTP_DATE_ENVVAR, date_str);

            let delay_secs = state.delay_seconds.to_string();
            tracing::debug!(delay_seconds = delay_secs, "DELAY_SECONDS RESET");
            std::env::set_var(RETRY_AFTER_DELAY_SECONDS_ENVVAR, delay_secs);
        }
        RetryAfterParams {
            http_date: None,
            delay_seconds: Some(delay_seconds),
        } => {
            let date_str = RETRY_AFTER_HTTP_DATE_DEFAULT;
            tracing::debug!(http_date = date_str, "HTTP_DATE RESET");
            std::env::set_var(RETRY_AFTER_HTTP_DATE_ENVVAR, date_str);

            let delay_secs = delay_seconds.to_string();
            tracing::debug!(delay_seconds = delay_secs, "DELAY_SECONDS SET");
            std::env::set_var(RETRY_AFTER_DELAY_SECONDS_ENVVAR, delay_seconds.to_string());
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
    use crate::{legacy_service::app, state::State};
    use axum::{
        body::{Body, HttpBody},
        http::{Request, StatusCode},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_ready_test() {
        let state = Arc::new(State::new(None).await.unwrap());
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
        let state = Arc::new(State::new(None).await.unwrap());
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
}
