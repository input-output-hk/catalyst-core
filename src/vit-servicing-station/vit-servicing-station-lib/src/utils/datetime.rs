use time::OffsetDateTime;

pub fn unix_timestamp_to_datetime(timestamp: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(timestamp).expect("invalid timestamp")
}
