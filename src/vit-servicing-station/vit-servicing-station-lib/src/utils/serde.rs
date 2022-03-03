use crate::db::models::vote_options::VoteOptions;
use crate::utils::datetime::unix_timestamp_to_datetime;
use serde::de::Visitor;
use serde::{ser::Error, Deserialize, Deserializer, Serializer};
use std::fmt;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

// this warning should be disable here since the interface for this function requires
// the first argument to be passed by value
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_unix_timestamp_as_rfc3339<S: Serializer>(
    timestamp: &i64,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let datetime = unix_timestamp_to_datetime(*timestamp);
    serializer.serialize_str(
        &datetime
            .format(&Rfc3339)
            .map_err(|e| S::Error::custom(format!("Could not serialize date: {}", e)))?,
    )
}

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
            OffsetDateTime::parse(value, &Rfc3339).map_err(|e| E::custom(format!("{}", e)))
        }
    }

    deserializer
        .deserialize_str(Rfc3339Deserializer())
        .map(|datetime| datetime.unix_timestamp())
}

pub fn serialize_bin_as_str<S: Serializer>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
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

// this warning should be disable here since the interface for this function requires
// the first argument to be passed by value
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn serialize_i64_as_str<S: Serializer>(data: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&data.to_string())
}

pub fn deserialize_i64_from_str<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    struct I64Deserializer();

    impl<'de> Visitor<'de> for I64Deserializer {
        type Value = i64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a compatible i64 number or string with i64 format")
        }

        fn visit_str<E>(self, value: &str) -> Result<i64, E>
        where
            E: serde::de::Error,
        {
            value
                .parse()
                .map_err(|e| E::custom(format!("Error parsing {} to i64: {}", value, e)))
        }
    }
    deserializer.deserialize_str(I64Deserializer())
}

pub fn deserialize_vote_options_from_string<'de, D>(
    deserializer: D,
) -> Result<VoteOptions, D::Error>
where
    D: Deserializer<'de>,
{
    struct VoteOptionsDeserializer();

    impl<'de> Visitor<'de> for VoteOptionsDeserializer {
        type Value = VoteOptions;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("A coma separated values are needed")
        }

        fn visit_str<E>(self, value: &str) -> Result<VoteOptions, E>
        where
            E: serde::de::Error,
        {
            Ok(VoteOptions::parse_coma_separated_value(value))
        }
    }

    deserializer.deserialize_str(VoteOptionsDeserializer())
}

pub fn deserialize_truthy_falsy<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let truthy_value = <&str>::deserialize(deserializer)?;
    Ok(matches!(
        truthy_value.to_lowercase().as_ref(),
        "x" | "1" | "true"
    ))
}
