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

#[test]
fn cip15_test_vector_hashes_correctly() {
    const HASH: &str = "a3d63f26cd94002443bc24f24b0a150f2c7996cd3a3fd247248de396faea6a5f";

    let SignedRegistration { registration, .. } = cip15::vector();
    let cbor = registration_to_cbor(&registration);
    let cbor_bytes = cbor_to_bytes(cbor);
    let cbor_hex = hex::encode(&cbor_bytes);

    assert_eq!(cbor_hex, HASH);
}

#[test]
fn fails_if_empty_delegations() {
    let db = make_db();
    let ctx = default_context(&db);

    let mut reg = cip15::vector();
    reg.registration.voting_power_source = VotingPowerSource::Delegated(vec![]);

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::EmptyDelegations);
}

#[test]
fn fails_if_stake_key_invalid_type() {
    let db = make_db();
    let ctx = default_context(&db);

    let mut reg = cip15::vector();
    let stake_key_bytes = [0; 32];
    reg.registration.stake_key = StakeKeyHex(PublicKeyHex::from_bytes(stake_key_bytes));

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

    reg.registration.stake_key = StakeKeyHex(PublicKeyHex::from_bytes(bytes));

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
