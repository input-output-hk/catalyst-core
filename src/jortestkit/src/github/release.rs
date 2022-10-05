use crate::github::{Asset, Release};
use os_info::Type as OsType;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

mod serializer {
    use serde::{self, de, ser, Deserialize, Deserializer, Serializer};
    use time::format_description::well_known::Rfc3339;
    use time::OffsetDateTime;

    pub fn serialize<S>(date: &Option<OffsetDateTime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => {
                let s = date.format(&Rfc3339).map_err(ser::Error::custom)?;
                serializer.serialize_some(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<OffsetDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let maybe_str = Option::<String>::deserialize(deserializer)?;
        match maybe_str {
            Some(s) => {
                let date = OffsetDateTime::parse(&s, &Rfc3339).map_err(de::Error::custom)?;
                Ok(Some(date))
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseDto {
    tag_name: String,
    #[serde(with = "serializer")]
    published_at: Option<OffsetDateTime>,
    assets: Vec<AssetDto>,
    prerelease: bool,
    draft: bool,
}

#[allow(clippy::from_over_into)]
impl Into<Release> for ReleaseDto {
    fn into(self) -> Release {
        let assets = Asset::assets_from_dtos(
            self.assets
                .into_iter()
                .filter(|x| x.is_generic())
                .collect::<Vec<_>>(),
        );
        Release {
            version: self.tag_name.clone(),
            released_date: self.published_at.map(|dt| dt.into()),
            releases_per_os: assets.iter().cloned().map(|x| (x.os_type(), x)).collect(),
            prerelease: self.prerelease,
            draft: self.draft,
        }
    }
}

impl ReleaseDto {
    pub fn tag_name(self) -> String {
        self.tag_name
    }

    pub fn published_at(&self) -> Option<&OffsetDateTime> {
        self.published_at.as_ref()
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

    fn is_generic(&self) -> bool {
        self.name.contains("generic")
    }

    pub fn download_url(&self) -> String {
        self.browser_download_url.clone()
    }

    pub fn name(&self) -> String {
        self.name.to_string()
    }
}
