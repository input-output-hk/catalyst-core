use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct JobParameters {
    #[serde(rename = "slot-id")]
    pub slot_id: u64,
    pub threshold: u64,
}
