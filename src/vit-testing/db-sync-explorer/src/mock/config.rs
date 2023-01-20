use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub providers: Providers,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Providers {
    BlockFrost,
}

impl Default for Providers {
    fn default() -> Self {
        Providers::BlockFrost
    }
}
