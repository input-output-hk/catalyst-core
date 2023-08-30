//! Command line and environment variable settings for the service
//!
use crate::logger::{LogFormat, LogLevel, LOG_FORMAT_DEFAULT, LOG_LEVEL_DEFAULT};
use clap::Args;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env;
use std::net::SocketAddr;
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

    /// Logging format
    #[clap(long, default_value = LOG_FORMAT_DEFAULT)]
    pub log_format: LogFormat,

    /// Logging level
    #[clap(long, default_value = LOG_LEVEL_DEFAULT)]
    pub log_level: LogLevel,
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
    use std::time::Duration;

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
            "https://github.com/input-output-hk/catalyst-core/issues/new?template=bug_report.md&title=Hello%2C%20World%21%20How%20are%20you%3F"
        );
    }
}
