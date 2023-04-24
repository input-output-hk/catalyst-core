use crate::types::utils::serialize_option_datetime_as_rfc3339;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventId(pub i32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectId(pub i32);

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ProposalSummary {
    pub id: EventId,
    pub name: String,
    pub summary: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    pub starts: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    pub ends: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_option_datetime_as_rfc3339"
    )]
    pub reg_checked: Option<DateTime<Utc>>,
    #[serde(rename = "final")]
    pub is_final: bool,
}
