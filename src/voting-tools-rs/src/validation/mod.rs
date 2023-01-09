use blake2::digest::{Update, VariableOutput};
use cardano_serialization_lib::chain_crypto::ed25519::{Pub, Sig};
use cardano_serialization_lib::chain_crypto::{
     AsymmetricPublicKey, Ed25519, VerificationAlgorithm,
};
use ciborium::cbor;

use ciborium::value::Value as Cbor;
use serde_json::Value;

use crate::data::{Registration, SignedRegistration};
use crate::Signature;
use error::ValidationError;
use validity::Validate;

mod error;

impl Validate for SignedRegistration {
    type Error = ValidationError;

    fn is_valid(&self) -> Result<(), Self::Error> {
        let SignedRegistration {
            registration,
            signature,
        } = self;

        let bytes = registration_as_cbor(registration);
        
        todo!()
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

fn extract_signature(Signature { inner }: &Signature) -> Result<Sig, ValidationError> {
    let sig = hex::decode(inner.as_str())?;
    let sig = Ed25519::signature_from_bytes(&sig)?;
    Ok(sig)
}

fn extract_key_from_registration(
    Registration { stake_key, .. }: &Registration,
) -> Result<Pub, ValidationError> {
    let key = hex::decode(stake_key.as_str())?;
    let key = Ed25519::public_from_binary(&key)?;
    Ok(key)
}

fn registration_as_cbor(
    Registration {
        voting_power_source,
        stake_key,
        rewards_addr,
        nonce,
        purpose,
    }: Registration,
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
    }).unwrap()
}
