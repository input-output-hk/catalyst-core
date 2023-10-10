//! Command line and environment variable settings for the service
//!
use crate::logger::{LogLevel, LOG_LEVEL_DEFAULT};
use chrono::{DateTime, Utc};
use clap::Args;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::env;
use std::net::{IpAddr, SocketAddr};
use tracing::log::error;
use url::Url;

// Default setting for JORM MOCK Timeout
#[cfg(feature = "jorm-mock")]
use crate::state::jorm_mock::JormState;
#[cfg(feature = "jorm-mock")]
use std::time::Duration;

/// Default address to start service on.
const ADDRESS_DEFAULT: &str = "0.0.0.0:3030";

/// Default Github repo owner
const GITHUB_REPO_OWNER_DEFAULT: &str = "input-output-hk";

/// Default Github repo name
const GITHUB_REPO_NAME_DEFAULT: &str = "catalyst-core";

/// Default Github issue template to use
const GITHUB_ISSUE_TEMPLATE_DEFAULT: &str = "bug_report.md";

/// Default CLIENT_ID_KEY used in development.
const CLIENT_ID_KEY_DEFAULT: &str = "3db5301e-40f2-47ed-ab11-55b37674631a";

/// Default API_HOSTNAME/S used in production.  This can be a single hostname, or a list of them.
const API_HOSTNAMES_DEFAULT: &str = "https://api.prod.projectcatalyst.io";

/// Default API_URL_PREFIX used in development.
const API_URL_PREFIX_DEFAULT: &str = "/api";

/// The name of the env var that specifies the delay-seconds in the 'Retry-After' header of 503
/// responses.
pub(crate) const RETRY_AFTER_HTTP_DATE_ENVVAR: &str = "RETRY_AFTER_HTTP_DATE";

/// The default http-date in the 'Retry-After' header of 503 responses.
pub(crate) const RETRY_AFTER_HTTP_DATE_DEFAULT: &str = "1970-01-01T00:00:00Z";

/// The default delay-seconds in the 'Retry-After' header of 503 responses.
pub(crate) const RETRY_AFTER_DELAY_SECONDS_DEFAULT: u64 = 120;
/// The name of the env var that specifies the delay-seconds in the 'Retry-After' header of 503
/// responses.
pub(crate) const RETRY_AFTER_DELAY_SECONDS_ENVVAR: &str = "RETRY_AFTER_DELAY_SECONDS";

#[derive(Args, Clone)]
pub struct Settings {
    /// Server binding address
    #[clap(long, default_value = ADDRESS_DEFAULT)]
    pub address: SocketAddr,

    /// Server binding address
    #[clap(long, default_value = Option::None)]
    pub metrics_address: Option<SocketAddr>,

    /// Url to the postgres event db
    #[clap(long, env)]
    pub database_url: String,

    /// Logging level
    #[clap(long, default_value = LOG_LEVEL_DEFAULT)]
    pub log_level: LogLevel,

    #[clap(long, env, default_value = "120")]
    pub delay_seconds: u64,
}

/// An environment variable read as a string.
pub(crate) struct StringEnvVar(String);

/// An environment variable read as a string.
impl StringEnvVar {
    /// Read the env var from the environment.
    ///
    /// If not defined, read from a .env file.
    /// If still not defined, use the default.
    ///
    /// # Arguments
    ///
    /// * `var_name`: &str - the name of the env var
    /// * `default_value`: &str - the default value
    ///
    /// # Returns
    ///
    /// * Self - the value
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// #use cat_data_service::settings::StringEnvVar;
    ///
    /// let var = StringEnvVar::new("MY_VAR", "default");
    /// assert_eq!(var.as_str(), "default");
    /// ```
    fn new(var_name: &str, default_value: &str) -> Self {
        dotenv().ok();
        let value = env::var(var_name).unwrap_or_else(|_| default_value.to_owned());
        Self(value)
    }

    /// Get the read env var as a str.
    ///
    /// # Returns
    ///
    /// * &str - the value
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Deserialize, Debug)]
pub(crate) struct RetryAfterParams {
    #[serde(rename = "http-date")]
    pub(crate) http_date: Option<DateTime<Utc>>,
    #[serde(rename = "delay-seconds")]
    pub(crate) delay_seconds: Option<u64>,
}

impl Default for RetryAfterParams {
    fn default() -> Self {
        RetryAfterParams {
            http_date: Some(RETRY_AFTER_HTTP_DATE_DEFAULT.parse().unwrap()),
            delay_seconds: Some(RETRY_AFTER_DELAY_SECONDS_DEFAULT),
        }
    }
}

impl RetryAfterParams {
    /// Set `RETRY_AFTER_HTTP_DATE` env var.
    pub fn delay_seconds_set_var(delay_secs: u64) {
        let delay_secs = delay_secs.to_string();
        std::env::set_var(RETRY_AFTER_DELAY_SECONDS_ENVVAR, &delay_secs);
        tracing::debug!(
            delay_seconds = delay_secs,
            "{RETRY_AFTER_DELAY_SECONDS_ENVVAR:} ENV set"
        );
    }
    /// Set `RETRY_AFTER_HTTP_DATE` env var.
    pub fn http_date_set_var(date_str: &str) {
        let date_str = date_str.to_string();
        std::env::set_var(RETRY_AFTER_HTTP_DATE_ENVVAR, &date_str);
        tracing::debug!(
            http_date = date_str,
            "{RETRY_AFTER_HTTP_DATE_ENVVAR:} ENV set"
        );
    }
    /// Return the header value string from env.
    pub fn header_value_from_env(delay_reset: u64) -> String {
        // Check the current value of the `RETRY_AFTER_HTTP_DATE` env var,
        // this is needed because the value can modified  by the
        // '/health/retry-after' endpoint.
        if let Ok(http_date_str) = std::env::var(RETRY_AFTER_HTTP_DATE_ENVVAR) {
            if http_date_str != RETRY_AFTER_HTTP_DATE_DEFAULT {
                if let Ok(http_date) = http_date_str.parse::<DateTime<Utc>>() {
                    if http_date > Utc::now() {
                        // If `http_date` is in the future, return 503 with the `http-date` in the
                        // `Retry-After` header.
                        return format!("{}", http_date.format("%a, %d %b %Y %H:%M:%S GMT"));
                    } else {
                        // If `http_date` has past, return 503 with the initial settings `delay-seconds` in the
                        // `Retry-After` header.
                        Self::http_date_set_var(RETRY_AFTER_HTTP_DATE_DEFAULT);
                        Self::delay_seconds_set_var(delay_reset);
                        return delay_reset.to_string();
                    }
                }
            }
        }

        // Check the current value of the `RETRY_AFTER_DELAY_SECONDS` env var,
        // this is needed because the value can modified  by the
        // '/health/retry-after' endpoint.
        if let Ok(delay) = std::env::var(RETRY_AFTER_DELAY_SECONDS_ENVVAR) {
            if let Ok(delay_seconds) = delay.parse::<u64>() {
                delay_seconds.to_string()
            } else {
                delay_reset.to_string()
            }
        } else {
            delay_reset.to_string()
        }
    }
}

// Lazy intialization of all env vars which are not command line parameters.
// All env vars used by the application should be listed here and all should have a default.
// The default for all NON Secret values should be suitable for Production, and NOT development.
// Secrets however should only be used with the default value in development.
lazy_static! {
    /// The github repo owner
    pub(crate) static ref GITHUB_REPO_OWNER: StringEnvVar = StringEnvVar::new("GITHUB_REPO_OWNER", GITHUB_REPO_OWNER_DEFAULT);

    /// The github repo name
    pub(crate) static ref GITHUB_REPO_NAME: StringEnvVar = StringEnvVar::new("GITHUB_REPO_NAME", GITHUB_REPO_NAME_DEFAULT);

    /// The github issue template to use
    pub(crate) static ref GITHUB_ISSUE_TEMPLATE: StringEnvVar = StringEnvVar::new("GITHUB_ISSUE_TEMPLATE", GITHUB_ISSUE_TEMPLATE_DEFAULT);

    /// The client id key used to anonymize client connections.
    pub(crate) static ref CLIENT_ID_KEY: StringEnvVar = StringEnvVar::new("CLIENT_ID_KEY", CLIENT_ID_KEY_DEFAULT);

    /// A List of servers to provideThe client id key used to anonymize client connections.
    pub(crate) static ref API_HOSTNAMES: StringEnvVar = StringEnvVar::new("API_HOSTNAMES", API_HOSTNAMES_DEFAULT);

    /// The Basepath the API is served at.
    pub(crate) static ref API_URL_PREFIX: StringEnvVar = StringEnvVar::new("API_URL_PREFIX", API_URL_PREFIX_DEFAULT);


}

/// Transform a string list of hostnames into a vec of hostnames.
/// Default to the service address if none specified.
///
fn string_to_api_hostnames(addr: &SocketAddr, hosts: &str) -> Vec<String> {
    fn invalid_hostname(hostname: &str) -> String {
        error!("Invalid hostname for API: {}", hostname);
        String::new()
    }

    let configured_hosts: Vec<String> = hosts
        .split(',')
        .map(|s| {
            let url = Url::parse(s.trim());
            match url {
                Ok(url) => {
                    // Get the scheme, and if its empty, use http
                    let scheme = url.scheme();

                    let port = url.port();

                    // Rebuild the scheme + hostname
                    match url.host() {
                        Some(host) => {
                            let host = host.to_string();
                            if host.is_empty() {
                                invalid_hostname(s)
                            } else {
                                match port {
                                    Some(port) => {
                                        format! {"{scheme}://{host}:{port}"}
                                        //scheme.to_owned() + "://" + &host + ":" + &port.to_string()
                                    }
                                    None => {
                                        format! {"{scheme}://{host}"}
                                    }
                                }
                            }
                        }
                        None => invalid_hostname(s),
                    }
                }
                Err(_) => invalid_hostname(s),
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    // If there are no hostnames, just use the address of the service.
    if configured_hosts.is_empty() {
        // If the Socket Address is the "catchall" address, then use localhost.
        if match addr.ip() {
            IpAddr::V4(ipv4) => ipv4.is_unspecified(),
            IpAddr::V6(ipv6) => ipv6.is_unspecified(),
        } {
            let port = addr.port();
            vec![format! {"http://localhost:{port}"}]
        } else {
            vec![format! {"http://{addr}"}]
        }
    } else {
        configured_hosts
    }
}

/// Get a list of all hostnames to serve the API on.
///
/// Used by the `OpenAPI` Documentation to point to the correct backend.
/// Take a list of [scheme://] + hostnames from the env var and turns it into
/// a lits of strings.
///
/// Hostnames are taken from the `API_HOSTNAMES` environment variable.
/// If that is not set, `addr` is used.
pub(crate) fn get_api_hostnames(addr: &SocketAddr) -> Vec<String> {
    string_to_api_hostnames(addr, API_HOSTNAMES.as_str())
}

// Jorm cleanup timeout is only used if feature is enabled.
#[cfg(feature = "jorm-mock")]
lazy_static! {
    /// The jorm mock timeout, only used if feature is enabled.
    pub(crate) static ref JORM_CLEANUP_TIMEOUT: Duration = {
        dotenv().ok();
        let value = match env::var("JORM_CLEANUP_TIMEOUT") {
            Ok(v) => match v.parse::<u64>() {
                Ok(v) => Some(v),
                Err(e) => {
                    // The default is fine if we can not parse.  Just report the error, and continue.
                    tracing::error!("Failed to parse JORM_CLEANUP_TIMEOUT: {}. Using Default.", e);
                    None
                }
            }
            Err(_) => None // Not an error if not set, just default it.
        };

        match value {
            Some(value) => Duration::from_secs(value * 60),
            None => JormState::CLEANUP_TIMEOUT
        }
    };
}

/// Generate a github issue url with a given title
///
/// ## Arguments
///
/// * `title`: &str - the title to give the issue
///
/// ## Returns
///
/// * String - the url
///
/// ## Example
///
/// ```rust,no_run
/// # use cat_data_service::settings::generate_github_issue_url;
/// assert_eq!(
///     generate_github_issue_url("Hello, World! How are you?"),
///     "https://github.com/input-output-hk/catalyst-core/issues/new?template=bug_report.md&title=Hello%2C%20World%21%20How%20are%20you%3F"
/// );
/// ```
pub(crate) fn generate_github_issue_url(title: &str) -> Option<Url> {
    let path = format!(
        "https://github.com/{}/{}/issues/new",
        GITHUB_REPO_OWNER.as_str(),
        GITHUB_REPO_NAME.as_str()
    );

    match Url::parse_with_params(
        &path,
        &[
            ("template", GITHUB_ISSUE_TEMPLATE.as_str()),
            ("title", title),
        ],
    ) {
        Ok(url) => Some(url),
        Err(e) => {
            error!(err = e.to_string(); "Failed to generate github issue url");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn github_repo_name_default() {
        assert_eq!(GITHUB_REPO_NAME.as_str(), GITHUB_REPO_NAME_DEFAULT);
    }

    #[test]
    fn github_repo_name_set() {
        env::set_var("GITHUB_REPO_NAME", "test");
        assert_eq!(GITHUB_REPO_NAME.as_str(), GITHUB_REPO_NAME_DEFAULT);
    }

    #[test]
    fn generate_github_issue_url() {
        let title = "Hello, World! How are you?";
        assert_eq!(
            super::generate_github_issue_url(title).unwrap().as_str(),
            "https://github.com/input-output-hk/catalyst-core/issues/new?template=bug_report.md&title=Hello%2C+World%21+How+are+you%3F"
        );
    }

    #[test]
    fn configured_hosts_default() {
        let configured_hosts = get_api_hostnames(&SocketAddr::from(([127, 0, 0, 1], 8080)));
        assert_eq!(
            configured_hosts,
            vec!["https://api.prod.projectcatalyst.io"]
        );
    }

    #[test]
    fn configured_hosts_set_multiple() {
        let configured_hosts = string_to_api_hostnames(
            &SocketAddr::from(([127, 0, 0, 1], 8080)),
            "http://api.prod.projectcatalyst.io , https://api.dev.projectcatalyst.io:1234",
        );
        assert_eq!(
            configured_hosts,
            vec![
                "http://api.prod.projectcatalyst.io",
                "https://api.dev.projectcatalyst.io:1234"
            ]
        );
    }

    #[test]
    fn configured_hosts_set_multiple_one_invalid() {
        let configured_hosts = string_to_api_hostnames(
            &SocketAddr::from(([127, 0, 0, 1], 8080)),
            "not a hostname , https://api.dev.projectcatalyst.io:1234",
        );
        assert_eq!(
            configured_hosts,
            vec!["https://api.dev.projectcatalyst.io:1234"]
        );
    }

    #[test]
    fn configured_hosts_set_empty() {
        let configured_hosts =
            string_to_api_hostnames(&SocketAddr::from(([127, 0, 0, 1], 8080)), "");
        assert_eq!(configured_hosts, vec!["http://127.0.0.1:8080"]);
    }

    #[test]
    fn configured_hosts_set_empty_undefined_address() {
        let configured_hosts = string_to_api_hostnames(&SocketAddr::from(([0, 0, 0, 0], 7654)), "");
        assert_eq!(configured_hosts, vec!["http://localhost:7654"]);
    }
}
