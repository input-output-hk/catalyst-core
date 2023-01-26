use proptest::prelude::Rng;

use crate::data::{
    arbitrary::arbitrary_signed_registration, Nonce, PubKey, RewardsAddress, SignedRegistration,
    TxId, VotingKeyHex,
};

/// CIP-15 test vectors
pub mod cip15;


/// Generate a random signed registration
///
/// This may not be fully valid (i.e. stake address may not be in exactly the right format)
pub fn generate_signed_registration(mut rng: impl Rng) -> SignedRegistration {
    let nonce = Nonce(rng.next_u64());

    let mut rewards_address = vec![0u8; 100];
    rng.fill_bytes(&mut rewards_address);
    let rewards_address = RewardsAddress(rewards_address.into());

    let mut voting_key = [0; 32];
    rng.fill_bytes(&mut voting_key);
    let voting_key = VotingKeyHex(PubKey(voting_key));

    let mut stake_secret = [0; 32];
    rng.fill_bytes(&mut stake_secret);

    let tx_id = TxId(rng.next_u64());

    arbitrary_signed_registration(nonce, rewards_address, voting_key, stake_secret, tx_id)
}

#[cfg(test)]
mod tests {
    use crate::{data::Registration, Signature};

    use serde_json::{from_value, json, Value};

    fn cip_15_test_vector() -> (Value, Value) {
        let reg = json!({
            "1": "0036ef3e1f0d3f5989e2d155ea54bdb2a72c4c456ccb959af4c94868f473f5a0",
            "2": "86870efc99c453a873a16492ce87738ec79a0ebd064379a62e2c9cf4e119219e",
            "3": "e0ae3a0a7aeda4aea522e74e4fe36759fca80789a613a58a4364f6ecef",
            "4": 1234,
        });

        let sig = json!({
            "1": "6c2312cd49067ecf0920df7e067199c55b3faef4ec0bce1bd2cfb99793972478c45876af2bc271ac759c5ce40ace5a398b9fdb0e359f3c333fe856648804780e",
        });

        (reg, sig)
    }

    #[test]
    fn can_deserialize_cip15_test_vector() {
        let (reg, sig) = cip_15_test_vector();
        let _reg: Registration = from_value(reg).unwrap();
        let _sig: Signature = from_value(sig).unwrap();
    }
}
