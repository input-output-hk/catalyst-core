mod release;

use os_info::Type as OsType;
pub use release::{AssetDto, ReleaseDto};
use reqwest::header::USER_AGENT;
use semver::Version;
use std::collections::HashMap;
use std::time::SystemTime;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum GitHubApiError {
    #[error("could not deserialize response")]
    CannotDeserialize(#[from] serde_json::Error),
    #[error("could not send reqeuest")]
    RequestError(#[from] reqwest::Error),
    #[error("cannot find release with version: {0}")]
    CannotFindReleaseWithVersion(String),
    #[error("API rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Clone, Debug)]
pub struct Release {
    version: String,
    released_date: SystemTime,
    releases_per_os: HashMap<OsType, AssetDto>,
    prerelease: bool,
}

impl Release {
    pub fn get_release_for_os(&self, os_type: OsType) -> Option<AssetDto> {
        let compacted_os_type = self.compact_os_types(os_type);
        self.releases_per_os().get(&compacted_os_type).cloned()
    }

    pub fn assets(&self) -> Vec<AssetDto> {
        self.releases_per_os.values().cloned().collect()
    }

    /// narrow linux distribution to linux type
    #[allow(clippy::all)]
    fn compact_os_types(&self, os_type: OsType) -> OsType {
        match os_type {
            OsType::Android => OsType::Android,
            OsType::Macos => OsType::Macos,
            OsType::Redox => OsType::Redox,
            OsType::Unknown => OsType::Unknown,
            OsType::Windows => OsType::Windows,
            _ => OsType::Linux,
        }
    }

    pub fn releases_per_os(&self) -> &HashMap<OsType, AssetDto> {
        &self.releases_per_os
    }

    pub fn version_str(&self) -> String {
        self.version.clone()
    }

    pub fn version(&self) -> Version {
        Version::parse(Self::without_first(&self.version_str())).unwrap()
    }

    fn without_first(string: &str) -> &str {
        string
            .char_indices()
            .nth(1)
            .and_then(|(i, _)| string.get(i..))
            .unwrap_or("")
    }

    pub fn prerelease(&self) -> bool {
        self.prerelease
    }
}

pub struct CachedReleases {
    inner: Vec<Release>,
}

impl CachedReleases {
    pub fn new(inner: Vec<Release>) -> Self {
        Self { inner }
    }

    pub fn get_asset_for_current_os_by_version(
        &self,
        version: String,
    ) -> Result<Option<AssetDto>, GitHubApiError> {
        let info = os_info::get();
        match self.inner.iter().find(|x| *x.version == version) {
            None => Err(GitHubApiError::CannotFindReleaseWithVersion(version)),
            Some(release) => Ok(release.get_release_for_os(info.os_type())),
        }
    }
}

impl<'a> IntoIterator for &'a CachedReleases {
    type Item = &'a Release;
    type IntoIter = std::slice::Iter<'a, Release>;

    fn into_iter(self) -> std::slice::Iter<'a, Release> {
        self.inner.iter()
    }
}

pub struct GitHubApi {
    base_url: String,
}

impl Default for GitHubApi {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubApi {
    pub fn for_crate<S: Into<String>>(base_url: S) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn new() -> Self {
        Self {
            base_url: "https://api.github.com/repos/input-output-hk/jormungandr".to_string(),
        }
    }

    fn get(&self, path: &str) -> Result<reqwest::blocking::Response, GitHubApiError> {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(&format!("{}/{}", self.base_url, path))
            .header(USER_AGENT, "request")
            .send()
            .map_err(GitHubApiError::RequestError)?;
        if resp.headers().get("X-RateLimit-Remaining") == Some(0.into()).as_ref() {
            return Err(GitHubApiError::RateLimitExceeded);
        }

        Ok(resp)
    }

    pub fn describe_releases(&self) -> Result<CachedReleases, GitHubApiError> {
        let response_text = self.get("releases")?.text()?;
        let releases: Vec<ReleaseDto> =
            serde_json::from_str(&response_text).map_err(GitHubApiError::CannotDeserialize)?;
        Ok(CachedReleases::new(
            releases
                .iter()
                .cloned()
                .map(|release| release.into())
                .collect(),
        ))
    }
}
