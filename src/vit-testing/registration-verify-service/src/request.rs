use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Request {
    pub qr: Vec<u8>,
    pub pin: String,
    pub expected_funds: u64,
    pub slot_no: u64,
}