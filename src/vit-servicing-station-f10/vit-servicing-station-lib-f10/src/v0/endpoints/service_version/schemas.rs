use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ServiceVersion {
    pub service_version: String,
}
