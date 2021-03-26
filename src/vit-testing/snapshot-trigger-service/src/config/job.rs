use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub struct JobParameters {
    #[serde(rename = "slot-no")]
    pub slot_no: Option<u64>,
    pub threshold: u64,
}
