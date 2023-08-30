//! This module contains generic re-usable responses with a 4xx response code.
//!

use poem_extensions::OneResponse;
use poem_openapi::payload::Json;
use poem_openapi::types::Example;
use poem_openapi::Object;
use url::Url;
use uuid::Uuid;

use crate::settings::generate_github_issue_url;

#[derive(Debug, Object)]
#[oai(example, skip_serializing_if_is_none)]
/// Response payload to a Bad request.
pub(crate) struct ServerErrorPayload {
    /// Unique ID of this Server Error so that it can be located easily for debugging.
    pub id: Uuid,
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
/// Internal Server Error
pub(crate) struct ServerError(Json<ServerErrorPayload>);

impl ServerError {
    /// Create a new Server Error Response.
    pub(crate) fn new(msg: Option<String>) -> Self {
        Self(Json(ServerErrorPayload::new(msg)))
    }

    /// Get the id of this Server Error.
    pub(crate) fn id(&self) -> Uuid {
        self.0.id
    }
}

#[derive(OneResponse)]
#[oai(status = 503)]
/// The service is not available.
pub(crate) struct ServiceUnavailable;
