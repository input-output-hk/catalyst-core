use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Request {
    pub payment_skey: String,
    pub payment_vkey: String,
    pub stake_skey: String,
    pub stake_vkey: String,
}
