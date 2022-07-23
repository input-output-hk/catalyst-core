use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JobParameters {
    #[serde(rename = "slot-no")]
    pub slot_no: Option<u64>,
    pub tag: Option<String>,
}

impl JobParameters {
    pub fn daily() -> Self {
        Self {
            slot_no: None,
            tag: Some("daily".to_string()),
        }
    }

    pub fn fund<S: Into<String>>(fund: S) -> Self {
        Self {
            slot_no: None,
            tag: Some(fund.into()),
        }
    }
}
