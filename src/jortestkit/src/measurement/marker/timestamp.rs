use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use std::{
    fmt,
    str::FromStr,
    time::{Duration, SystemTime},
};

#[derive(Clone, Copy)]
pub struct Timestamp(SystemTime);

impl Default for Timestamp {
    fn default() -> Self {
        Self::new()
    }
}

impl Timestamp {
    pub fn new() -> Self {
        Timestamp::from(SystemTime::now())
    }

    pub fn duration_since(&self, earlier: &Timestamp) -> Duration {
        let system_time: SystemTime = (*earlier).into();
        self.0.duration_since(system_time).unwrap()
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed().unwrap()
    }
}

impl From<SystemTime> for Timestamp {
    fn from(from: SystemTime) -> Self {
        Timestamp(from)
    }
}
#[allow(clippy::from_over_into)]
impl Into<SystemTime> for Timestamp {
    fn into(self) -> SystemTime {
        self.0
    }
}
impl FromStr for Timestamp {
    type Err = time::error::Parse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dt = OffsetDateTime::parse(s, &Rfc3339)?;
        Ok(Timestamp(dt.into()))
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
