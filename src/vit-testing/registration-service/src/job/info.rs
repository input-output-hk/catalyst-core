use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct JobOutputInfo {
    pub slot_no: u64,
    pub funds: u64,
}
