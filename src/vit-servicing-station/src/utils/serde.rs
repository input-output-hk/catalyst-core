use crate::utils::datetime::unix_timestamp_to_datetime;
use chrono::{DateTime, Utc};
use serde::de::Visitor;
use serde::{Deserializer, Serializer};
use std::fmt;

// this warning should be disable here since the interface for this function requires
// the first argument to be passed by value
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_unix_timestamp_as_rfc3339<S: Serializer>(
    timestamp: &i64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let datetime = unix_timestamp_to_datetime(*timestamp);
    serializer.serialize_str(&datetime.to_rfc3339())
}

pub fn deserialize_unix_timestamp_from_rfc3339<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct RFC3339Deserializer();

    impl<'de> Visitor<'de> for RFC3339Deserializer {
        type Value = DateTime<Utc>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("An rfc3339 compatible string is needed")
        }

        fn visit_str<E>(self, value: &str) -> Result<DateTime<Utc>, E>
        where
            E: serde::de::Error,
        {
            let date: DateTime<Utc> = DateTime::parse_from_rfc3339(value)
                .map_err(|e| E::custom(format!("{}", e)))?
                .with_timezone(&Utc);
            Ok(date)
        }
    }

    deserializer
        .deserialize_str(RFC3339Deserializer())
        .map(|datetime| datetime.timestamp())
}

pub fn serialize_bin_as_string<S: Serializer>(
    data: &[u8],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&String::from_utf8(data.to_vec()).unwrap())
}

pub fn deserialize_string_as_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct VecU8Deserializer();

    impl<'de> Visitor<'de> for VecU8Deserializer {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A compatible utf8 string is needed")
        }

        fn visit_str<E>(self, value: &str) -> Result<Vec<u8>, E>
        where
            E: serde::de::Error,
        {
            let vec = value.as_bytes().to_vec();
            Ok(vec)
        }
    }

    deserializer.deserialize_str(VecU8Deserializer())
}
