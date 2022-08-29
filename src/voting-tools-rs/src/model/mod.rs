use cardano_serialization_lib::address::NetworkInfo;
use microtype::microtype;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
#[derive(Debug, Clone, PartialEq)]
pub enum Delegations {
    Legacy(String),
    Delegated(Vec<(String, u32)>),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
    pub delegations: Delegations,
    pub rewards_address: RewardsAddr,
    pub stake_public_key: StakePubKey,
    pub voting_power: VotingPower,
    pub voting_purpose: VotingPurpose,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegoSignature {
    #[serde(rename = "1")]
    pub signature: Signature,
}

#[derive(Debug, Clone)]
pub struct Rego {
    pub tx_id: TxId,
    pub metadata: RegoMetadata,
    pub signature: RegoSignature,
}

// Create newtype wrappers for better type safety
microtype! {
    #[derive(Debug, PartialEq, Clone)]
    #[string]
    pub String {
        DbName,
        DbUser,
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
        DbPass,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
    #[int]
    pub u64 {
        #[cfg_attr(test, derive(test_strategy::Arbitrary))]
        SlotNo,
        VotingPower,
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
