use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AdditionalServices {
    pub explorer: bool,
    pub archive: Option<String>,
}
