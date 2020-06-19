use chrono::{DateTime, Utc};
use serde::de::Visitor;
use serde::{Deserializer, Serializer};
use std::fmt;

pub fn serialize_datetime_as_rfc3339<S: Serializer>(
    datetime: &DateTime<Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&datetime.to_rfc3339())
}

pub fn deserialize_datetime_from_rfc3339<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
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

    deserializer.deserialize_str(RFC3339Deserializer())
}

pub fn serialize_bin_as_string<S: Serializer>(
    data: &[u8],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&String::from_utf8(data.to_vec()).unwrap())
}
