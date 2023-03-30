use chrono::{DateTime, Utc};
use serde::Serializer;

pub fn serialize_datetime_as_rfc3339<S: Serializer>(
    time: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&time.to_rfc3339())
}
