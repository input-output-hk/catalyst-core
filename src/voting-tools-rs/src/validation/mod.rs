use blake2::digest::{Update, VariableOutput};
use cardano_serialization_lib::chain_crypto::ed25519::{Pub, Sig};
use cardano_serialization_lib::chain_crypto::{
    AsymmetricPublicKey, Ed25519, Verification, VerificationAlgorithm,
};
use cardano_serialization_lib::NetworkId;
use ciborium::cbor;

use ciborium::value::Value as Cbor;

use crate::data::crypto::PublicKeyHex;
use crate::data::{Registration, SignedRegistration, StakeKeyHex, VotingPurpose};
use crate::error::RegistrationError;
use crate::{DataProvider, Signature, SignatureHex, VotingPowerSource};
use validity::Validate;

/// Helper macro that calls a function that returns `Result<(), RegistrationError>`, and pushes it
/// to `errors` if it errors, and does nothing if it succeeds
macro_rules! handle {
    ($errors:expr, $f:expr) => {
        match $f {
            Ok(()) => {}
            Err(e) => $errors.push(e),
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ValidationCtx<'a> {
    pub db: &'a dyn DataProvider,
    /// Used for validating addresses
    pub network_id: NetworkId,
    pub expected_voting_purpose: VotingPurpose,
}

impl Validate for SignedRegistration {
    type Context<'a> = ValidationCtx<'a>;
    type Error = RegistrationError;

    fn is_valid(&self, ctx: Self::Context<'_>) -> Result<(), Self::Error> {
        let SignedRegistration {
            registration,
            signature,
            tx_id: _,
        } = self;

        // don't use with_capacity here, we want to avoid allocating in the happy path (i.e. no
        // errors)
        let mut errs = vec![];

        handle!(
            errs,
            validate_voting_power(&registration.voting_power_source)
        );

        let StakeKeyHex(PublicKeyHex(key)) = &registration.stake_key;
        let Signature {
            inner: SignatureHex(signature),
        } = signature;

        let cbor = registration_as_cbor(registration);
        let bytes = cbor_to_bytes(cbor);
        let hash_of_bytes = blake2b_256(&bytes);

        match Ed25519::verify_bytes(&key, &signature, &hash_of_bytes) {
            Verification::Success => Ok(()),
            Verification::Failed => Err(RegistrationError::MismatchedSignature),
        }
    }
}

fn validate_voting_power(source: &VotingPowerSource) -> Result<(), RegistrationError> {
    match source {
        VotingPowerSource::Delegated(vec) if vec.len() == 0 => {
            Err(RegistrationError::EmptyDelegations)
        }
        _ => Ok(()),
    }
}

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
    use crate::data::TxId;

    use super::*;

    #[test]
    fn can_serialize_cbor() {
        let cbor = cbor!({
            1 => "hello",
            2 => 123,
            3 => [1, 2, 3, 4],
        })
        .unwrap();
        cbor_to_bytes(cbor); // doesn't panic
    }

    fn good_registration() -> SignedRegistration {
        // let signed_registration = SignedRegistration {
        //     tx_id: TxId(1),
        //     registration: Registration {
        //         voting_power_source: VotingPowerSource::Delegated(vec![]),
        //
        //     }
        //     signature: todo!(),
        //
        // };

        todo!()
    }

    #[test]
    fn fails_if_empty_delegations() {
        let mut reg = good_registration();
        reg.registration.voting_power_source = VotingPowerSource::Delegated(vec![]);
    }
}
