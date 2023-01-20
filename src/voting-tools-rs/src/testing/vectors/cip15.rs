use serde_json::{json, Value};

use crate::{
    data::{
        Nonce, PublicKeyHex, Registration, RewardsAddress, SignedRegistration, StakeKeyHex, TxId,
        VotingPurpose,
    },
    Signature, SignatureHex, VotingPowerSource,
};

/// CIP-15 test vector voting key
pub const VOTING_KEY: &str = "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0";
/// CIP-15 test vector stake key
pub const STAKE_KEY: &str = "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e";
/// CIP-15 test vector reward address
pub const REWARD_ADDRESS: &str = "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef";
/// CIP-15 test vector signature
pub const SIGNATURE: &str = "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e";
/// CIP-15 test vector nonce
pub const NONCE: u64 = 1234;

/// The full CIP-15 test vector
pub fn vector() -> SignedRegistration {
    SignedRegistration {
        tx_id: TxId(1), // not provided in the test vector
        registration: Registration {
            voting_power_source: VotingPowerSource::direct_from_hex(VOTING_KEY).unwrap(),
            stake_key: StakeKeyHex(PublicKeyHex::from_hex(STAKE_KEY).unwrap()),
            rewards_address: RewardsAddress::from_hex(REWARD_ADDRESS).unwrap(),
            nonce: Nonce(NONCE),
            voting_purpose: VotingPurpose::CATALYST,
        },
        signature: Signature {
            inner: SignatureHex::from_hex(SIGNATURE).unwrap(),
        },
    }
}

/// The json we expect to get from db-sync corresponding to the CIP-15 test vector
pub fn vector_json() -> Value {
    json!({
        "61284": {
            "1": VOTING_KEY,
            "2": STAKE_KEY,
            "3": REWARD_ADDRESS,
            "4": NONCE,
        },
        "61285": {
            "1": SIGNATURE,
        }
    })
}
