use serde_json::{json, Value};

use crate::{
    data::{Nonce, PubKey, Registration, RewardsAddress, SignedRegistration, StakeKeyHex, TxId},
    Sig, Signature, VotingKey,
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

// The following 2 consts should be identical, but they're not. This appears to be a bug in the
// cip15 test vector. For now we're going to just ignore those tests, since they **should** be
// rejected as invalid

/// The result of:
///  - cbor encoding the metadata
///  - then hashing it with `blake2b_256`
pub const METADATA_HASH_HEX: &str = "a119ef64a40158200036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a002582086870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e03581de0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef041904d2";

/// Computed by putting the metadata into the cbor playground
pub const METADATA_HASH_HEX_FROM_PLAYGROUND: &str = "a119ef64a401584230783030333665663365316630643366353938396532643135356561353462646232613732633463343536636362393539616634633934383638663437336635613002584230783836383730656663393963343533613837336131363439326365383737333865633739613065626430363433373961363265326339636634653131393231396503583c307865306165336130613761656461346165613532326537346534666533363735396663613830373839613631336135386134333634663665636566041904d2";

/// The private key corresponding to [`STAKE_KEY`]
pub const STAKE_PRIVATE_KEY: &str =
    "f5beaeff7932a4164d270afde7716067582412e8977e67986cd9b456fc082e3a";

/// The full CIP-15 test vector
#[must_use]
pub fn vector() -> SignedRegistration {
    SignedRegistration {
        tx_id: TxId(1), // not provided in the test vector
        slot: 0,
        registration: Registration {
            voting_key: VotingKey::direct_from_hex(VOTING_KEY).unwrap(),
            stake_key: StakeKeyHex(PubKey::from_hex(STAKE_KEY).unwrap()),
            rewards_address: RewardsAddress::from_hex(REWARD_ADDRESS).unwrap(),
            nonce: Nonce(NONCE),
            voting_purpose: None,
        },
        signature: Signature {
            inner: Sig::from_hex(SIGNATURE).unwrap(),
        },
        staked_ada: None,
        stake_key_hash: vec![0; 29],
    }
}

/// The json we expect to get from db-sync corresponding to the CIP-15 test vector
#[must_use]
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

#[cfg(test)]
mod tests {
    use cardano_serialization_lib::chain_crypto::{AsymmetricKey, AsymmetricPublicKey, Ed25519};

    use super::*;

    // this test is mainly verifying that cip15 test vectors are correct
    #[test]
    fn signature_given_is_correct() {
        let secret_bytes = hex::decode(STAKE_PRIVATE_KEY).unwrap();
        let secret = Ed25519::secret_from_binary(&secret_bytes).unwrap();

        let public_bytes = hex::decode(STAKE_KEY).unwrap();
        let public = Ed25519::public_from_binary(&public_bytes).unwrap();

        let public_from_secret = Ed25519::compute_public(&secret);

        // cardano_serialization_lib types don't implement Debug so we can't use assert_eq!
        // lmao
        if public != public_from_secret {
            panic!()
        }
    }
}
