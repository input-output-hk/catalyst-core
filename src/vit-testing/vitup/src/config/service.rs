use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Service {
    pub version: String,
    pub https: bool,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            version: "3.6".to_string(),
            https: false,
        }
    }
}
