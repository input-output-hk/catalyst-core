mod live_get;
mod ready_get;
mod started_get;

use crate::service::generic::responses::resp_5xx::ServiceUnavailable;
use crate::service::generic::responses::{resp_2xx::NoContent, resp_5xx::ServerError};
use crate::service::generic::tags::ApiTags;

use poem_openapi::OpenApi;

use poem_extensions::response;

pub(crate) struct HealthApi;

#[OpenApi(prefix_path = "/health", tag = "ApiTags::Health")]
impl HealthApi {
    #[oai(path = "/started", method = "get")]
    /// Service Started
    ///
    /// This endpoint is used to determine if the service has started properly
    /// and is able to serve requests.
    ///
    /// ## Responses
    ///
    /// * 204 No Content - Service is Started and can serve requests.
    /// * 500 Server Error - If anything within this function fails unexpectedly.
    /// * 503 Service Unavailable - Service has not started, do not send other requests yet.
    ///
    /// ## Note
    ///
    /// *This endpoint is for internal use of the service deployment infrastructure.
    /// It may not be exposed publicly.*
    ///
    async fn started_get(
        &self,
    ) -> response! {
           204: NoContent,
           500: ServerError,
           503: ServiceUnavailable,
       } {
        started_get::endpoint().await
    }

    #[oai(path = "/ready", method = "get")]
    /// Service Ready
    ///
    /// This endpoint is used to determine if the service is ready and able to serve requests.
    ///
    /// ## Responses
    ///
    /// * 204 No Content - Service is Ready and can serve requests.
    /// * 500 Server Error - If anything within this function fails unexpectedly.
    /// * 503 Service Unavailable - Service has not ready, requests to other
    /// endpoints should not be sent until the service becomes ready.
    ///
    /// ## Note
    ///
    /// *This endpoint is for internal use of the service deployment infrastructure.
    /// It may not be exposed publicly.*
    ///
    async fn ready_get(
        &self,
    ) -> response! {
           204: NoContent,
           500: ServerError,
           503: ServiceUnavailable,
       } {
        ready_get::endpoint().await
    }

    #[oai(path = "/live", method = "get")]
    /// Service Live
    ///
    /// This endpoint is used to determine if the service is live.
    ///
    /// ## Responses
    ///
    /// * 204 No Content - Service is Live and can serve requests.
    /// * 500 Server Error - If anything within this function fails unexpectedly.
    /// * 503 Service Unavailable - Service is not Live.
    ///
    /// ## Note
    ///
    /// *This endpoint is for internal use of the service deployment infrastructure.
    /// It may not be exposed publicly. Refer to []*
    ///
    async fn live_get(
        &self,
    ) -> response! {
           204: NoContent,
           500: ServerError,
           503: ServiceUnavailable,
       } {
        live_get::endpoint().await
    }
}
