use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Request {
    pub source: Source,
    pub expected_funds: u64,
    pub threshold: u64,
    pub slot_no: Option<u64>,
    pub tag: Option<String>,
}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Request")
            .field(&self.expected_funds)
            .field(&self.threshold)
            .field(&self.slot_no)
            .finish()
    }
}

#[derive(PartialEq, Eq, Clone, Deserialize, Serialize, Debug)]
pub enum Source {
    Qr {
        #[serde(skip_serializing)]
        content: Vec<u8>,
        pin: String,
    },
    PublicKeyBytes(#[serde(skip_serializing)] Vec<u8>),
}
