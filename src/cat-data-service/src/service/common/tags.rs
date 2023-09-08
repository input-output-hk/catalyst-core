//! OpenAPI Tags we need to classify the endpoints.
//!
use poem_openapi::Tags;

#[derive(Tags)]
pub(crate) enum ApiTags {
    // Health Endpoints
    Health,
    // Test Endpoints (Not part of the API)
    Test,
    TestTag2,
}
