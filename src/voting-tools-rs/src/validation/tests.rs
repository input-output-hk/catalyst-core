use std::collections::BTreeMap;

use mainnet_lib::InMemoryDbSync;
use test_strategy::proptest;
use validity::Failure;

use crate::{
    data::{RewardsAddress, TxId},
    test_api::MockDbProvider,
    vectors::cip15,
};

use super::*;

fn make_db() -> MockDbProvider {
    MockDbProvider::from(InMemoryDbSync::empty())
}

fn default_context(db: &MockDbProvider) -> ValidationCtx {
    ValidationCtx {
        db,
        network_id: NetworkId::Testnet, // makes bit twiddling easier, since it's all 0s
        expected_voting_purpose: VotingPurpose::CATALYST,
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
    cbor_to_bytes(cbor); // doesn't panic
}

// taken from the cip 15 test vector
const HASH_BYTES_HEX: &str = "a3d63f26cd94002443bc24f24b0a150f2c7996cd3a3fd247248de396faea6a5f";
const METADATA_CBOR_HEX_BYTES: &str = "a119ef64a401982000183618ef183e181f0d183f1859188918e218d1185518ea185418bd18b218a7182c184c1845186c18cb1895189a18f418c91848186818f4187318f518a0029820188618870e18fc189918c4185318a8187318a11864189218ce18871873188e18c7189a0e18bd061843187918a6182e182c189c18f418e118191821189e03981d18e018ae183a0a187a18ed18a418ae18a5182218e7184e184f18e31867185918fc18a807188918a61318a5188a1843186418f618ec18ef041904d2";

#[test]
fn blake2b_256_works() {
    let bytes = hex::decode(METADATA_CBOR_HEX_BYTES).unwrap();
    let hash = blake2b_256(&bytes);
    let hash_hex = hex::encode(&hash);

    assert_eq!(hash_hex, HASH_BYTES_HEX);
}

#[test]
fn cip15_test_vector_hashes_correctly() {
    let SignedRegistration { registration, .. } = cip15::vector();
    let cbor = registration.to_cbor();
    let cbor_bytes = cbor_to_bytes(cbor);
    let cbor_bytes_hex = hex::encode(&cbor_bytes);

    assert_eq!(cbor_bytes_hex, METADATA_CBOR_HEX_BYTES);

    let hash = blake2b_256(&cbor_bytes);
    let cbor_hex = hex::encode(&hash);

    assert_eq!(cbor_hex, HASH_BYTES_HEX);
}

#[test]
fn fails_if_empty_delegations() {
    let db = make_db();
    let ctx = default_context(&db);

    let mut reg = cip15::vector();
    reg.registration.voting_power_source = VotingPowerSource::Delegated(BTreeMap::new());

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::EmptyDelegations);
}

#[test]
fn fails_if_stake_key_invalid_type() {
    let db = make_db();
    let ctx = default_context(&db);

    let mut reg = cip15::vector();
    let stake_key_bytes = [0; 32];
    reg.registration.stake_key = StakeKeyHex(PubKey::from_bytes(stake_key_bytes));

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::StakeKeyWrongType(0))
}

#[test]
fn fails_if_stake_key_wrong_network_id() {
    let db = make_db();
    let mut ctx = default_context(&db);

    ctx.network_id = NetworkId::Mainnet;

    let mut reg = cip15::vector();
    let leading_byte = 0b11110000; // type 15, testnet
    let mut bytes = [0; 32];
    bytes[0] = leading_byte;

    reg.registration.stake_key = StakeKeyHex(PubKey::from_bytes(bytes));

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();

    assert_eq!(
        error,
        RegistrationError::StakeKeyWrongNetwork {
            expected: NetworkId::Mainnet,
            actual: Some(NetworkId::Testnet),
        }
    )
}

#[test]
fn cip15_test_vector_succeeds() {
    let db = make_db();
    let ctx = default_context(&db);

    let reg = cip15::vector();

    let _valid = reg.validate_with(ctx).unwrap();
}
