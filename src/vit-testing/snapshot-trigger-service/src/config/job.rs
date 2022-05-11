use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JobParameters {
    #[serde(rename = "slot-no")]
    pub slot_no: Option<u64>,
    pub tag: Option<String>,
}
