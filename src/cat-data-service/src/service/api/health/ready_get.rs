//! Implementation of the GET /health/ready endpoint

use std::sync::Arc;

use crate::service::common::responses::resp_2xx::NoContent;
use crate::service::common::responses::resp_5xx::{server_error, ServerError, ServiceUnavailable};
use crate::settings::RetryAfterParams;
use crate::state::State;
use poem::web::Data;
use poem_extensions::response;
use poem_extensions::UniResponse::{T204, T500, T503};

pub(crate) type AllResponses = response! {
    204: NoContent,
    500: ServerError,
    503: ServiceUnavailable,
};

/// # GET /health/ready
///
/// Readiness endpoint.
///
/// Kubernetes (and others) use this endpoint to determine if the service is
/// able to service requests.
///
/// In this service, readiness is guaranteed if this endpoint is reachable.
/// So, it will always just return 204.
///
/// In theory it can also return 503 if for some reason a temporary circumstance
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
/// * 500 Server Error - If anything within this function fails unexpectedly. (Possible but unlikely)
/// * 503 Service Unavailable - Service is not ready, do not send other requests.
pub(crate) async fn endpoint(state: Data<&Arc<State>>) -> AllResponses {
    // otherwise everything seems to be A-OK
    match state.event_db.schema_version_check().await {
        Ok(current_ver) => {
            tracing::info!(schema_version = current_ver, "verified schema version");
            T204(NoContent)
        }
        Err(e) => {
            tracing::warn!(error = e.to_string(), "could not verify schema version");
            // Only return error if it is not a connection timeout
            match e {
                event_db::error::Error::ConnectionTimeout => T503(ServiceUnavailable(
                    RetryAfterParams::header_value_from_env(state.delay_seconds),
                )),
                err => T500(server_error!("{}", err.to_string())),
            }
        }
    }
}
