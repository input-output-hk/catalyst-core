use cardano_serialization_lib::chain_crypto::ed25519::{Pub, Sig};

pub struct Registration {
    pub metadata: Metadata,
    pub signature: Signature,
}

pub struct Metadata {
    pub voting_key: Pub,
    pub stake_pub: Pub,
    pub reward_address: Sig,
    nonce: u64,
}

pub struct Signature {
    inner: Sig,
}
