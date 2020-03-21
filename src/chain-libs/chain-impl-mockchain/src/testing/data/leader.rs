use crate::key::BftLeaderId;
use chain_crypto::{Ed25519, KeyPair, SecretKey};
use quickcheck::{Arbitrary, Gen};
use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct LeaderPair {
    pub leader_id: BftLeaderId,
    pub leader_key: SecretKey<Ed25519>,
}

impl Debug for LeaderPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LeaderPair")
            .field("proposal", &self.id())
            .finish()
    }
}

impl LeaderPair {
    pub fn new(leader_id: BftLeaderId, leader_key: SecretKey<Ed25519>) -> Self {
        LeaderPair {
            leader_id,
            leader_key,
        }
    }

    pub fn id(&self) -> BftLeaderId {
        self.leader_id.clone()
    }

    pub fn key(&self) -> SecretKey<Ed25519> {
        self.leader_key.clone()
    }
}

impl Arbitrary for LeaderPair {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        LeaderPair {
            leader_id: BftLeaderId::arbitrary(g),
            leader_key: KeyPair::<Ed25519>::arbitrary(g).private_key().clone(),
        }
    }
}
