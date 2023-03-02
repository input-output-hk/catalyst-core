use std::collections::BTreeMap;

use cardano_serialization_lib::chain_crypto::{AsymmetricKey, SigningAlgorithm};
use ciborium::cbor;
use test_strategy::proptest;
use validity::Failure;

use crate::{
    data::PubKey,
    vectors::cip15::{self, METADATA_HASH_HEX, SIGNATURE, STAKE_KEY, STAKE_PRIVATE_KEY},
    Sig,
};

use super::*;

fn test_ctx() -> ValidationCtx {
    ValidationCtx {
        network_id: NetworkId::Testnet, // makes bit twiddling easier, since it's all 0s
        ..ValidationCtx::default()
    }
}

#[test]
fn can_serialize_cbor() {
    let cbor = cbor!({
        1 => "hello",
        2 => 123,
        3 => [1, 2, 3, 4],
    })
    .unwrap();
    cbor_to_bytes(&cbor); // doesn't panic
}

#[test]
#[ignore = "the cip15 test vector incorrectly encodes itself, so the hash is invalid"]
fn gets_correct_cbor_hash_of_registration() {
    let reg = cip15::vector();
    let cbor = reg.registration.to_cbor();
    let cbor_bytes = cbor_to_bytes(&cbor);
    let cbor_hex = hex::encode(cbor_bytes);

    assert_eq!(cbor_hex.len(), METADATA_HASH_HEX.len()); // easier debugging
    assert_eq!(cbor_hex, METADATA_HASH_HEX); // this hash is provided by cip15, but is incorrect
}

#[test]
fn fails_if_empty_delegations() {
    let ctx = test_ctx();

    let mut reg = cip15::vector();
    reg.registration.voting_power_source = VotingPowerSource::Delegated(BTreeMap::new());

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::EmptyDelegations);
}

#[test]
fn fails_if_stake_key_invalid_type() {
    let ctx = test_ctx();

    let mut reg = cip15::vector();
    let stake_key_bytes = vec![0; 32];
    reg.registration.stake_key = StakeKeyHex(PubKey(stake_key_bytes));

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::StakeKeyWrongType(0));
}

#[test]
fn fails_if_stake_key_wrong_network_id() {
    let mut ctx = test_ctx();
    ctx.network_id = NetworkId::Mainnet;

    let mut reg = cip15::vector();
    let leading_byte = 0b1111_0000; // type 15, testnet
    let mut bytes = vec![0; 32];
    bytes[0] = leading_byte;

    reg.registration.stake_key = StakeKeyHex(PubKey(bytes));

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();

    assert_eq!(
        error,
        RegistrationError::StakeKeyWrongNetwork {
            expected: NetworkId::Mainnet,
            actual: Some(NetworkId::Testnet),
        }
    );
}

/// Sign a registration with the key provided in the CIP 15 vector
fn compute_sig_from_registration(reg: &Registration) -> Signature {
    let data = reg.to_cbor();
    let data_bytes = cbor_to_bytes(&data);

    let secret_bytes = hex::decode(STAKE_PRIVATE_KEY).unwrap();
    let secret_key = Ed25519::secret_from_binary(&secret_bytes).unwrap();

    let public_bytes = hex::decode(STAKE_KEY).unwrap();
    let public_key = Ed25519::public_from_binary(&public_bytes).unwrap();

    assert!(
        Ed25519::compute_public(&secret_key) != public_key,
        "inconsistent secret/public key pair"
    );

    let signature = Ed25519::sign(&secret_key, &data_bytes);
    let signature_bytes = signature.as_ref();

    Signature {
        inner: Sig(signature_bytes.try_into().unwrap()),
    }
}

#[test]
#[ignore = "the cip15 test vector incorrectly encodes itself, so the signature is invalid"]
fn cip15_test_vector_correct_sig() {
    let reg = cip15::vector();

    let sig = compute_sig_from_registration(&reg.registration);
    let signature_bytes = sig.inner.0;
    let vector_signature_bytes = hex::decode(SIGNATURE).unwrap();

    assert_eq!(signature_bytes.to_vec(), vector_signature_bytes);
}

#[test]
#[ignore = "the cip15 test vector incorrectly encodes itself, so the signature is invalid"]
fn cip15_test_vector_succeeds() {
    let mut ctx = test_ctx();

    // the cip 15 test vector contains an invalid key, since it is type 8, so we disable some
    // checks
    ctx.validate_key_type = false;
    ctx.validate_network_id = false;

    let reg = cip15::vector();

    let _valid = reg.validate_with(ctx).unwrap();
}

#[proptest]
fn validates_valid_registrations(reg: SignedRegistration) {
    let mut ctx = test_ctx();

    ctx.validate_key_type = false;
    ctx.validate_network_id = false;

    let _valid = reg.validate_with(ctx).unwrap();
}
