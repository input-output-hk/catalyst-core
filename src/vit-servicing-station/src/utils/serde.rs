use chrono::{DateTime, Utc};
use serde::Serializer;

pub fn serialize_datetime_as_rfc3339<S: Serializer>(
    datetime: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&datetime.to_rfc3339())
}

pub fn serialize_bin_as_string<S: Serializer>(
    data: &[u8],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&String::from_utf8(data.to_vec()).unwrap())
}
