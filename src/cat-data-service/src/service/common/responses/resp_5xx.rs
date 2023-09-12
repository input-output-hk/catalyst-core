//! This module contains common and re-usable responses with a 4xx response code.
//!

use crate::settings::generate_github_issue_url;
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem_extensions::OneResponse;
use poem_openapi::payload::Json;
use poem_openapi::types::Example;
use poem_openapi::Object;
use url::Url;
use uuid::Uuid;

#[derive(Debug, Object)]
#[oai(example, skip_serializing_if_is_none)]
/// Response payload to a Bad request.
/*pub(crate)*/
struct ServerErrorPayload {
    /// Unique ID of this Server Error so that it can be located easily for debugging.
    id: Uuid,
    /// *Optional* SHORT Error message.
    /// Will not contain sensitive information, internal details or backtraces.
    msg: Option<String>,
    /// A URL to report an issue.
    issue: Option<Url>,
}

impl ServerErrorPayload {
    /// Create a new Server Error Response Payload.
    pub(crate) fn new(msg: Option<String>) -> Self {
        let id = Uuid::new_v4();
        let issue_title = format!("Internal Server Error - {id}");
        let issue = generate_github_issue_url(&issue_title);

        Self { id, msg, issue }
    }
}

impl Example for ServerErrorPayload {
    /// Example for the Server Error Payload.
    fn example() -> Self {
        Self::new(Some("Server Error".to_string()))
    }
}

#[derive(OneResponse)]
#[oai(status = 500)]
/// ## Internal Server Error
///
/// An internal server error occurred.
///
/// *The contents of this response should be reported to the projects issue tracker.*
pub struct ServerError(Json<ServerErrorPayload>);

impl ServerError {
    /// Create a new Server Error Response.
    pub fn new(msg: Option<String>) -> Self {
        Self(Json(ServerErrorPayload::new(msg)))
    }

    /// Get the id of this Server Error.
    pub fn id(&self) -> Uuid {
        self.0.id
    }
}

impl ResponseError for ServerError {
    fn status(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

#[derive(OneResponse, Debug)]
#[oai(status = 503)]
/// ## Service Unavailable
///
/// The service is not available.
///
/// *This is returned when the service either has not started,
/// or has become unavailable.*
///
/// #### NO DATA BODY IS RETURNED FOR THIS RESPONSE
pub struct ServiceUnavailable;
