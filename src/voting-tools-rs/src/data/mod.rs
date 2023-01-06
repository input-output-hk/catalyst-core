use serde::{Serialize, Deserialize};
use crypto::PublicKeyHex;

pub mod crypto;

/// The source of voting power for a given registration
///
/// The voting power can either come from: 
///  - a single wallet, OR
///  - a set of delegations
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Clone, PartialEq)]
pub enum VotingPowerSource {
    /// Direct voting
    ///
    /// The voting power comes from a single wallet
    Legacy(PublicKeyHex),

    /// Delegated one. Collection of catalyst identifiers joined it weights
    Delegated(Vec<(PublicKeyHex, u32)>),
}

/// A registration on Cardano in either CIP-15 or CIP-36 format
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Registration {
    #[serde(rename = "1")]
    pub voting_power_source: VotingPowerSource,
    #[serde(rename = "2")]
    pub stake_key: StakeKeyHex,
    #[serde(rename = "3")]
    pub rewards_addr: RewardsAddr,
    // note, this must be monotonically increasing. Typically, the current slot
    // number is used
    #[serde(rename = "4")]
    pub nonce: Nonce,  
    #[serde(rename = "5")]
    #[serde(default)]
    pub purpose: VotingPurpose,
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn voting_power_source_wire_format() {
        let legacy = json!("");
    }
}
