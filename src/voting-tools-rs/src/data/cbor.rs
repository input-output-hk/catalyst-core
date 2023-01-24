use ciborium::{cbor, value::Value};

use super::*;

impl Registration {
    pub fn to_cbor(&self) -> Value {
        let Self {
            voting_power_source,
            stake_key,
            rewards_address,
            nonce,
            voting_purpose,
        } = self;
        cbor!({
            61284 => {
                1 => voting_power_source.to_cbor(),
                2 => stake_key.to_cbor(),
                3 => rewards_address.to_cbor(),
                4 => nonce.to_cbor(),
            }
        })
        .unwrap()
    }
}

impl VotingPowerSource {
    pub fn to_cbor(&self) -> Value {
        match self {
            Self::Direct(key) => cbor!(key.to_bytes()).unwrap(),
            Self::Delegated(key) => todo!(),
        }
    }
}

impl StakeKeyHex {
    pub fn to_cbor(&self) -> Value {
        cbor!(self.0.to_bytes()).unwrap()
    }
}

impl RewardsAddress {
    pub fn to_cbor(&self) -> Value {
        cbor!(self.0).unwrap()
    }
}

impl Nonce {
    pub fn to_cbor(&self) -> Value {
        cbor!(self.0).unwrap()
    }
}
