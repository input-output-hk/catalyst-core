//! Implementation of the GET /health/live endpoint

use crate::service::generic::responses::resp_2xx::NoContent;
use crate::service::generic::responses::resp_5xx::{ServerError, ServiceUnavailable};

use poem_extensions::response;
use poem_extensions::UniResponse::T204;

/// # GET /health/live
///
/// Liveness endpoint.
///
/// Kubernetes (and others) use this endpoint to determine if the service is able
/// to keep running.
///
/// In this service, liveness is assumed unless there are multiple panics generated
/// by an endpoint in a short window.
///
/// ## Responses
///
/// * 204 No Content - Service is OK and can keep running.
/// * 500 Server Error - If anything within this function fails unexpectedly. (Possible but unlikely)
/// * 503 Service Unavailable - Service is possibly not running reliably.
#[allow(clippy::unused_async)]
pub(crate) async fn endpoint() -> response! {
       204: NoContent,
       500: ServerError,
       503: ServiceUnavailable,
   } {
    // TODO: Detect if too many panics have occurred in a defined window.
    // If so, return a 503
    // T503(ServiceUnavailable)

    // otherwise everything seems to be A-OK
    T204(NoContent)
}
