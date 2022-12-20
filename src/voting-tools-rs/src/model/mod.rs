use bigdecimal::BigDecimal;
use cardano_serialization_lib::address::NetworkInfo;
use microtype::microtype;
use serde::{Deserialize, Serialize};

/// Responsible to hold information about voting power assignment
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Clone, PartialEq)]
pub enum Delegations {
    /// Direct voting. String should contain catalyst identifier
    Legacy(String),
    /// Delegated one. Collection of catalyst identifiers joined it weights
    Delegated(Vec<(String, u32)>),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RegoMetadata {
    #[serde(rename = "1")]
    pub delegations: Delegations,
    #[serde(rename = "2")]
    pub stake_vkey: StakeVKey,
    #[serde(rename = "3")]
    pub rewards_addr: RewardsAddr,
    #[serde(rename = "4")]
    pub slot: SlotNo,
    #[serde(rename = "5")]
    #[serde(default)]
    pub purpose: VotingPurpose,
}

/// Single output element of voting tools calculations
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    /// registration content
    pub delegations: Delegations,
    /// mainnet rewards address
    pub rewards_address: RewardsAddr,
    /// stake public key
    pub stake_public_key: StakePubKey,
    /// voting power expressed in ada
    pub voting_power: BigDecimal,
    /// voting purpose. By default 0
    pub voting_purpose: VotingPurpose,
    /// registration transaction id
    pub tx_id: TxId,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RegoSignature {
    #[serde(rename = "1")]
    pub signature: Signature,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reg {
    pub tx_id: TxId,
    pub metadata: RegoMetadata,
    pub signature: RegoSignature,
}

// Create newtype wrappers for better type safety
microtype! {
    #[derive(Debug, PartialEq, Clone)]
    #[string]
    pub String {
        /// Database name
        DbName,
        /// Database user
        DbUser,
        /// Database host
        DbHost,
        RewardsAddr,
        StakeAddr,
        StakePubKey,
        Signature,
        #[derive(Hash, PartialOrd, Eq)]
        StakeVKey,
    }

    #[secret]
    #[string]
    pub String {
         /// Database password
        DbPass,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    #[int]
    pub u64 {
        #[cfg_attr(test, derive(test_strategy::Arbitrary))]
        SlotNo,
        VotingPurpose,
        TxId,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    #[int]
    pub u32 {
        TestnetMagic
    }
}

impl RewardsAddr {
    pub fn without_leading_0x(&self) -> Self {
        self.trim_start_matches("0x").into()
    }
}

pub fn network_info(testnet_magic: Option<TestnetMagic>) -> NetworkInfo {
    match testnet_magic {
        None => NetworkInfo::mainnet(),
        Some(TestnetMagic(magic)) => NetworkInfo::new(NetworkInfo::testnet().network_id(), magic),
    }
}

impl SlotNo {
    pub fn into_i64(self) -> color_eyre::eyre::Result<i64> {
        Ok(self.0.try_into()?)
    }
}
