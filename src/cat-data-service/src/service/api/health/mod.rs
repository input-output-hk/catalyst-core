//mod live_get;
//mod ready_get;

use crate::service::generic::responses::resp_5xx::ServiceUnavailable;
use crate::service::generic::responses::{resp_2xx::NoContent, resp_5xx::ServerError};

use poem_openapi::OpenApi;

use poem_extensions::{
    response,
    UniResponse::{T204, T503},
};

pub(crate) struct HealthApi;

#[OpenApi]
impl HealthApi {
    #[oai(path = "/health/ready", method = "get")]
    async fn health_get(
        &self,
    ) -> response! {
           204: NoContent,
           500: ServerError,
           503: ServiceUnavailable,
       } {
        T204(NoContent)
    }

    #[oai(path = "/health/live", method = "get")]
    async fn live_get(
        &self,
    ) -> response! {
           204: NoContent,
           500: ServerError,
           503: ServiceUnavailable,
       } {
        // Return No Content unless any endpoint panics.
        // If there are x panics in a time frame,  say the service is unavailable to force a restart.
        T503(ServiceUnavailable)
    }
}
