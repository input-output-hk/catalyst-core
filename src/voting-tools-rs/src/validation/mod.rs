use blake2::digest::{Update, VariableOutput};
use cardano_serialization_lib::chain_crypto::ed25519::{Pub, Sig};
use cardano_serialization_lib::chain_crypto::{
    AsymmetricPublicKey, Ed25519, Verification, VerificationAlgorithm,
};
use ciborium::cbor;

use ciborium::value::Value as Cbor;
use serde_json::Value;

use crate::data::crypto::PublicKeyHex;
use crate::data::{Registration, SignedRegistration, StakeKeyHex};
use crate::{Signature, SignatureHex};
use validity::Validate;

mod error;
pub use error::ValidationError;

impl Validate for SignedRegistration {
    type Error = ValidationError;

    fn is_valid(&self) -> Result<(), Self::Error> {
        let SignedRegistration {
            registration,
            signature,
            tx_id: _,
        } = self;

        let StakeKeyHex(PublicKeyHex(key)) = &registration.stake_key;
        let Signature { inner: SignatureHex(signature) } = signature;

        let cbor = registration_as_cbor(registration);
        let bytes = cbor_to_bytes(cbor);
        let hash_of_bytes = blake2b_256(&bytes);


        match Ed25519::verify_bytes(&key, &signature, &hash_of_bytes) {
            Verification::Success => Ok(()),
            Verification::Failed => Err(ValidationError::InvalidSignature),
        }
    }
}

// /// Validates whether a signature is valid for a given metadata
// ///
// /// ```
// /// # use serde_json::json;
// /// let metadata = json!({
// ///     "1": ""
// /// })
// /// ```
// pub fn validate(metadata: Value, signature: Value) -> bool {
// let Some(sig) = extract_signature(signature) else { return false; };
// let Some(key) = extract_key_from_metadata(metadata.clone()) else { return false; };
//
// let Some(cbor) = json_to_cbor(metadata) else { return false; };
// let mut bytes = Vec::<u8>::new();
//
// match ciborium::ser::into_writer(&cbor, &mut bytes) {
//     Ok(()) => {}
//     Err(_) => return false,
// };
//
// let hash = blake2b_256(&bytes);
//
// match Ed25519::verify_bytes(&key, &sig, &hash) {
//     Verification::Success => true,
//     Verification::Failed => false,
// }
// }

const HASH_SIZE: usize = 32;

/// Simple helper function to compute the blake2b_256 hash of a byte slice
fn blake2b_256(bytes: &[u8]) -> [u8; HASH_SIZE] {
    let mut hasher = blake2::Blake2bVar::new(HASH_SIZE).unwrap();
    hasher.update(bytes);
    let mut buf = [0u8; HASH_SIZE];
    hasher.finalize_variable(&mut buf).unwrap();
    buf
}

fn registration_as_cbor(
    Registration {
        voting_power_source,
        stake_key,
        rewards_address: rewards_addr,
        nonce,
        voting_purpose: purpose,
    }: &Registration,
) -> Cbor {
    // we do this manually because it's not really a 1:1 conversion and serde can't handle integer
    // keys
    cbor!({
        61284 => {
            1 => voting_power_source,
            2 => stake_key,
            3 => rewards_addr,
            4 => nonce,
        }
    })
    .unwrap()
}

fn cbor_to_bytes(cbor: Cbor) -> Vec<u8> {
    let mut bytes: Vec<u8> = vec![];
    ciborium::ser::into_writer(&cbor, &mut bytes).unwrap();
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_serialize_cbor() {
        let cbor = cbor!({
            1 => "hello",
            2 => 123,
            3 => [1, 2, 3, 4],
        }).unwrap();
        cbor_to_bytes(cbor); // doesn't panic
    }
}
