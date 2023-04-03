use core::fmt::Display;
use core::str::FromStr;

use serde::Serialize;
use thiserror::Error;

/// An identifier for a cardano network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum NetworkId {
    Mainnet,
    Testnet,
}

impl NetworkId {
    pub fn rewards_address_prefix(&self) -> String {
        match self {
            Self::Testnet => "e0".to_owned(),
            Self::Mainnet => "e1".to_owned(),
        }
    }
}

impl Display for NetworkId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        })
    }
}

impl FromStr for NetworkId {
    type Err = NetworkInfoFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mainnet" => Ok(Self::Mainnet),
            "testnet" => Ok(Self::Testnet),
            s => Err(NetworkInfoFromStrError(s.to_string())),
        }
    }
}
#[derive(Debug, Error)]
#[error("unknown variant: {0}")]
pub struct NetworkInfoFromStrError(String);
