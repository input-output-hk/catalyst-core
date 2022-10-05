use crate::key::BftLeaderId;
use chain_crypto::{Ed25519, KeyPair, SecretKey};
use quickcheck::{Arbitrary, Gen};
use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct LeaderPair {
    leader_key: SecretKey<Ed25519>,
}

impl Debug for LeaderPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LeaderPair")
            .field("leader id", &self.id())
            .finish()
    }
}

impl LeaderPair {
    pub fn new(leader_key: SecretKey<Ed25519>) -> Self {
        LeaderPair { leader_key }
    }

    pub fn id(&self) -> BftLeaderId {
        BftLeaderId(self.leader_key.to_public())
    }

    pub fn key(&self) -> SecretKey<Ed25519> {
        self.leader_key.clone()
    }
}

impl Arbitrary for LeaderPair {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        LeaderPair {
            leader_key: KeyPair::<Ed25519>::arbitrary(g).private_key().clone(),
        }
    }
}
