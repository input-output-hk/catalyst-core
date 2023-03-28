use serde::{ser::Error, Serializer};
use std::time::SystemTime;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

pub fn serialize_systemtime_as_rfc3339<S: Serializer>(
    time: &SystemTime,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let datetime: OffsetDateTime = time.clone().into();
    serializer.serialize_str(
        &datetime
            .format(&Rfc3339)
            .map_err(|e| S::Error::custom(format!("Could not serialize date: {}", e)))?,
    )
}
