use crate::types::utils::{serialize_datetime_as_rfc3339, serialize_option_datetime_as_rfc3339};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventId(pub i32);

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct EventSummary {
    pub id: EventId,
    pub name: String,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub starts: DateTime<Utc>,
    #[serde(serialize_with = "serialize_datetime_as_rfc3339")]
    pub ends: DateTime<Utc>,
    #[serde(rename = "final")]
    pub is_final: bool,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    pub reg_checked: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use serde_json::json;

    #[test]
    fn event_id_json_test() {
        let event_ids = vec![EventId(10), EventId(11), EventId(12)];
        let json = serde_json::to_value(&event_ids).unwrap();
        assert_eq!(json, json!([10, 11, 12]));

        let decoded: Vec<EventId> = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, event_ids);
    }

    #[test]
    fn event_summary_json_test() {
        let event_summary = EventSummary {
            id: EventId(1),
            name: "Fund 10".to_string(),
            starts: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            ends: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
            reg_checked: Some(DateTime::from_utc(
                NaiveDateTime::from_timestamp_opt(0, 0).unwrap(),
                Utc,
            )),
        };

        let json = serde_json::to_value(&event_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "name": "Fund 10",
                    "starts": "1970-01-01T00:00:00+00:00",
                    "ends": "1970-01-01T00:00:00+00:00",
                    "final": true,
                    "reg_checked": "1970-01-01T00:00:00+00:00",
                }
            )
        );

        let event_summary = EventSummary {
            id: EventId(1),
            name: "Fund 10".to_string(),
            starts: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            ends: DateTime::from_utc(NaiveDateTime::from_timestamp_opt(0, 0).unwrap(), Utc),
            is_final: true,
            reg_checked: None,
        };

        let json = serde_json::to_value(&event_summary).unwrap();
        assert_eq!(
            json,
            json!(
                {
                    "id": 1,
                    "name": "Fund 10",
                    "starts": "1970-01-01T00:00:00+00:00",
                    "ends": "1970-01-01T00:00:00+00:00",
                    "final": true,
                }
            )
        );
    }
}
