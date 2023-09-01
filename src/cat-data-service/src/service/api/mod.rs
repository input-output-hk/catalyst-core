//! Catalyst Data Service API Definition
//!
//! This defines all endpoints for the Catalyst Data Service API.
//! It however does NOT contain any processing for them, that is defined elsewhere.
use health::HealthApi;
use poem_openapi::{
    param::Query, payload::PlainText, ContactObject, LicenseObject, OpenApi, OpenApiService,
    ServerObject,
};
use std::net::SocketAddr;

use crate::settings::{get_api_hostnames, API_URL_PREFIX};

mod health;
mod test_endpoints;

/// The name of the API
const API_TITLE: &str = "Catalyst Data Service";

/// The version of the API
const API_VERSION: &str = "1.2.0";

/// Get the contact details for inquiring about the API
fn get_api_contact() -> ContactObject {
    ContactObject::new()
        .name("Project Catalyst Team")
        .email("contact@projectcatalyst.io")
        .url("https://projectcatalyst.io")
}

/// A summary describing the API
const API_SUMMARY: &str = "Project Catalyst Data Service API";

/// A long description of the API. Markdown is supported
const API_DESCRIPTION: &str = r#"# Catalyst Data Service API.

The Catalyst Data Service API provides realtime data for all prior, current and future Catalyst voting events.

TODO:

* Implement Permissionless Auth.
* Implement Replacement Functionality for GVC.
* Implement representative registration on main-chain, distinct from voter registration.
* Implement Voting API abstracting the Jormungandr API from public exposure.
* Implement Audit API's (Retrieve voting blockchain records,  registration/voting power audit and private tally audit.
* Implement API's needed to support posting Ideas/Proposals etc.Catalyst Data Service
"#;

/// Get the license details for the API
fn get_api_license() -> LicenseObject {
    LicenseObject::new("Apache 2.0")
        .url("https://www.apache.org/licenses/LICENSE-2.0")
        .identifier("Apache-2.0")
}

/// Get the terms of service for the API
const TERMS_OF_SERVICE: &str =
    "https://github.com/input-output-hk/catalyst-core/blob/main/book/src/98_CODE_OF_CONDUCT.md";

/// Combine all the API's into one
pub(crate) type OpenApiServiceT = OpenApiService<(Api, HealthApi), ()>;

/// The API
pub(crate) struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/hello", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        // API's, parameters and response types should NOT be defined in this file.
        // This should simply call the implementation.
        // No parameter or other processing should be done here.
        match name.0 {
            Some(name) => PlainText(format!("hello, {}!", name)),
            None => PlainText("hello!".to_string()),
        }
    }
}

pub(crate) fn mk_api(addr: &SocketAddr) -> OpenApiServiceT {
    let mut service = OpenApiService::new((Api, HealthApi), API_TITLE, API_VERSION)
        .contact(get_api_contact())
        .description(API_DESCRIPTION)
        .license(get_api_license())
        .summary(API_SUMMARY)
        .terms_of_service(TERMS_OF_SERVICE)
        .url_prefix(API_URL_PREFIX.as_str());

    // Add the Servers to the service.
    // There can be multiple.
    let server_hosts = get_api_hostnames();
    if server_hosts.len() == 0 {
        // This should be the actual hostname of the service.  But in the absence of that, the IP address/port will do.
        let server_host = format!("http://{}:{}", addr.ip(), addr.port());

        service = service.server(ServerObject::new(server_host));
    } else {
        for host in server_hosts {
            service = service.server(ServerObject::new(host));
        }
    }

    service
}
