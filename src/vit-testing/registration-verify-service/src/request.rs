use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Request {
    #[serde(skip_serializing)]
    pub qr: Vec<u8>,
    pub pin: String,
    pub expected_funds: u64,
    pub threshold: u64,
    pub slot_no: Option<u64>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Request")
            .field(&self.pin)
            .field(&self.expected_funds)
            .field(&self.threshold)
            .field(&self.slot_no)
            .finish()
    }
}
