use ciborium::{cbor, value::Value};
use itertools::Itertools;

use super::{Nonce, Registration, RewardsAddress, StakeKeyHex, VotingKey, VotingPurpose};

impl Registration {
    pub(crate) fn to_cbor(&self) -> Value {
        let Self {
            voting_key,
            stake_key,
            rewards_address,
            nonce,
            voting_purpose,
        } = self;
        match voting_purpose {
            None => cbor!({
                61284 => {
                    1 => voting_key.to_cbor(),
                    2 => stake_key.clone().to_cbor(),
                    3 => rewards_address.to_cbor(),
                    4 => nonce.to_cbor(),
                }
            }),
            Some(voting_purpose) => cbor!({

                61284 => {
                    1 => voting_key.to_cbor(),
                    2 => stake_key.clone().to_cbor(),
                    3 => rewards_address.to_cbor(),
                    4 => nonce.to_cbor(),
                    5 => voting_purpose.to_cbor(),
                }
            }),
        }
        .unwrap()
    }
}

impl VotingKey {
    pub(crate) fn to_cbor(&self) -> Value {
        match self {
            Self::Direct(key) => cbor!(key.0).unwrap(),
            Self::Delegated(map) => {
                let vec = map.iter().collect_vec();
                cbor!(vec).unwrap()
            }
        }
    }
}

impl StakeKeyHex {
    pub(crate) fn to_cbor(&self) -> Value {
        cbor!(self.0 .0).unwrap()
    }
}

impl RewardsAddress {
    pub(crate) fn to_cbor(&self) -> Value {
        cbor!(self.0).unwrap()
    }
}

impl Nonce {
    pub(crate) fn to_cbor(self) -> Value {
        cbor!(self.0).unwrap()
    }
}

impl VotingPurpose {
    pub(crate) fn to_cbor(self) -> Value {
        cbor!(self.0).unwrap()
    }
}
