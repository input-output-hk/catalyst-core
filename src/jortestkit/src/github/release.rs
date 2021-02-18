use crate::github::{Asset, Release};
use chrono::{DateTime, Utc};
use os_info::Type as OsType;
use serde::{Deserialize, Serialize};

mod serializer {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Utc.datetime_from_str(&s, FORMAT)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseDto {
    tag_name: String,
    #[serde(with = "serializer")]
    published_at: DateTime<Utc>,
    assets: Vec<AssetDto>,
    prerelease: bool,
}

impl Into<Release> for ReleaseDto {
    fn into(self) -> Release {
        let assets = Asset::assets_from_dtos(self.assets);
        Release {
            version: self.tag_name.clone(),
            released_date: self.published_at.into(),
            releases_per_os: assets.iter().cloned().map(|x| (x.os_type(), x)).collect(),
            prerelease: self.prerelease,
        }
    }
}

impl ReleaseDto {
    pub fn tag_name(self) -> String {
        self.tag_name
    }

    pub fn published_at(&self) -> &DateTime<Utc> {
        &self.published_at
    }

    pub fn assets(&self) -> &Vec<AssetDto> {
        &self.assets
    }

    pub fn prerelease(&self) -> bool {
        self.prerelease
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AssetDto {
    browser_download_url: String,
    name: String,
}

impl AssetDto {
    pub fn os_type(&self) -> OsType {
        if self.is_x86_64() && self.is_windows() {
            return OsType::Windows;
        } else if self.is_x86_64() && self.is_unix() {
            return OsType::Linux;
        } else if self.is_x86_64() && self.is_apple() {
            return OsType::Macos;
        }
        OsType::Unknown
    }

    fn is_x86_64(&self) -> bool {
        self.name.contains("x86_64")
    }

    fn is_windows(&self) -> bool {
        self.name.contains("windows")
    }

    fn is_apple(&self) -> bool {
        self.name.contains("apple")
    }

    fn is_unix(&self) -> bool {
        self.name.contains("linux")
    }

    pub fn download_url(&self) -> String {
        self.browser_download_url.clone()
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
