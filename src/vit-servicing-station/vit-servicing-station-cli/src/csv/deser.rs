use serde::de::Visitor;
use serde::Deserializer;
use std::fmt;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

pub fn deserialize_unix_timestamp_from_rfc3339<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct Rfc3339Deserializer();

    impl<'de> Visitor<'de> for Rfc3339Deserializer {
        type Value = OffsetDateTime;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("An rfc3339 compatible string is needed")
        }

        fn visit_str<E>(self, value: &str) -> Result<OffsetDateTime, E>
        where
            E: serde::de::Error,
        {
            match OffsetDateTime::parse(value, &Rfc3339) {
                Ok(date) => Ok(date),
                Err(original) => {
                    let timestamp = value.parse().map_err(|e| {
                        E::custom(format!(
                            "Cannot parse date, tried to parse as date time, but got: '{}', \
                    then tried as unix, but got: '{}'",
                            original, e
                        ))
                    })?;
                    OffsetDateTime::from_unix_timestamp(timestamp)
                        .map_err(|e| E::custom(format!("{}", e)))
                }
            }
        }
    }

    deserializer
        .deserialize_str(Rfc3339Deserializer())
        .map(|datetime| datetime.unix_timestamp())
}
