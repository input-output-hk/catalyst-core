use std::collections::BTreeMap;

use ciborium::cbor;
use mainnet_lib::InMemoryDbSync;
use validity::Failure;

use crate::{data::PubKey, test_api::MockDbProvider, vectors::cip15};

use super::*;

fn make_db() -> MockDbProvider {
    MockDbProvider::from(InMemoryDbSync::empty())
}

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
    let stake_key_bytes = [0; 32];
    reg.registration.stake_key = StakeKeyHex(PubKey(stake_key_bytes));

    let Failure { error, .. } = reg.validate_with(ctx).unwrap_err();
    assert_eq!(error, RegistrationError::StakeKeyWrongType(0))
}

#[test]
fn fails_if_stake_key_wrong_network_id() {
    let mut ctx = test_ctx();

    ctx.network_id = NetworkId::Mainnet;

    let mut reg = cip15::vector();
    let leading_byte = 0b11110000; // type 15, testnet
    let mut bytes = [0; 32];
    bytes[0] = leading_byte;

    reg.registration.stake_key = StakeKeyHex(PubKey(bytes));

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
    let mut ctx = test_ctx();

    // the cip 15 test vector contains an invalid key, since it is type 8, so we disable some
    // checks
    ctx.validate_key_type = false;
    ctx.validate_network_id = false;

    let reg = cip15::vector();

    let _valid = reg.validate_with(ctx).unwrap();
}
