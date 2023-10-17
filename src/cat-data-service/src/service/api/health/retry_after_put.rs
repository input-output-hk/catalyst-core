//! Implementation of the PUT /health/retry-after endpoint

use std::sync::Arc;

use crate::settings::RetryAfterParams;
use crate::state::State;
use crate::{
    service::common::responses::{
        resp_2xx::NoContent,
        resp_4xx::ApiValidationError,
        resp_5xx::{server_error, ServerError},
    },
    settings::RETRY_AFTER_HTTP_DATE_DEFAULT,
};
use poem::error::ParseQueryError;
use poem::{
    web::{Data, Query},
    Result,
};
use poem_extensions::response;
use poem_extensions::UniResponse::{T204, T400, T500};
use poem_openapi::payload::PlainText;

pub(crate) type AllResponses = response! {
    204: NoContent,
    400: ApiValidationError,
    500: ServerError,
};

/// # PUT /health/retry-after
///
/// Configuration of the "retry-after" for 503 responses endpoint.
///
/// During the service runtime, when disconnects to the database happen,
/// the appropriate response status code is 503, which includes a string
/// that is either `<delay-secods>` or `<http-date>`.
///
/// This endpoint accepts two optional query parameters:
///
/// * `http-date` is the UTC Datetime string, which if set, will be returned in all 503 responses.
///   Example:  `PUT /health/retry-after?http-date=2099-01-31T00:00:00Z`
/// * `delay-seconds` is number of seconds as a positive integer, which if set, will be returned in
///   all 503 response.
///   Example:  `PUT /health/retry-after?delay-seconds=120`
///
/// If none of the query parameters is present, then the header value will be reset
/// to the service's original settings.
///   Example:  `PUT /health/retry-after`
///
/// If no problems occur, this endpoint will always just return 204.
///
/// If query parameters are invalid, for example an invalid datetime strinn
/// In theory it can also return 500 if for some reason a temporary circumstance
/// is preventing this service from properly serving request.
///
/// An example could be the service has started a long and cpu intensive task
/// and is not able to properly service requests while it is occurring.
/// This would let the load balancer shift traffic to other instances of this
/// service that are ready.
///
/// ## Responses
///
/// * 204 No Content - Service is Ready to serve requests.
/// * 400 API Validation Error - If anything within this function fails unexpectedly. (Possible but unlikely)
/// * 500 Service  - Service is not ready, do not send other requests.
#[allow(clippy::unused_async)]
pub(crate) async fn endpoint(
    params: Result<Query<RetryAfterParams>>,
    state: Data<&Arc<State>>,
) -> AllResponses {
    match params {
        Ok(Query(params)) => {
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
            };
            T204(NoContent)
        }
        Err(err) if err.is::<ParseQueryError>() => {
            T400(ApiValidationError(PlainText(err.to_string())))
        }
        Err(err) => T500(server_error!("{}", err.to_string())),
    }
}
