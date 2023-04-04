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
    pub fn is_valid_rewards_address(&self, prefix: String) -> bool {
        if prefix.len() != 2 {
            return false;
        }
        // First nibble represents rewards address
        let rewards_address = prefix.chars().nth(0).unwrap();
        // second nibble represent network id
        let network_id = prefix.chars().nth(1).unwrap();

        // Valid addrs: 0x0?, 0x1?, 0x2?, 0x3?, 0x4?, 0x5?, 0x6?, 0x7?, 0xE?, 0xF?.
        let valid_addrs = 0..7;
        let addr = rewards_address.to_digit(16).unwrap();
        if !valid_addrs.contains(&addr) && addr != 14 && addr != 15 {
            error!("invalid rewards addr prefix {:?} {:?}", prefix, addr);
            return false;
        }

        let net_id = network_id.to_digit(10).unwrap();

        match self {
            Self::Testnet => {
                if net_id != 0 {
                    return false;
                }
            }
            Self::Mainnet => {
                if net_id != 1 {
                    return false;
                }
            }
        }

        true
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
