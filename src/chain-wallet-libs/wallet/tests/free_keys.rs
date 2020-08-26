mod utils;

use self::utils::State;
use chain_crypto::{bech32::Bech32, SecretKey};
use chain_impl_mockchain::{
    certificate::VoteCast,
    fragment::Fragment,
    value::Value,
    vote::{Choice, Payload},
};
use std::convert::TryInto;
use wallet::transaction::dump_free_utxo;

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const ACCOUNT: &str = include_str!("../../test-vectors/free_keys/key1.prv");
const UTXO1: &str = include_str!("../../test-vectors/free_keys/key2.prv");
const UTXO2: &str = include_str!("../../test-vectors/free_keys/key3.prv");
const WALLET_VALUE: Value = Value(10_000 + 1000);

/// test to recover a daedalus style address in the test-vectors block0
///
#[test]
fn free_keys1() {
    let builder = wallet::RecoveryBuilder::new();

    let builder = builder
        .account_secret_key(SecretKey::try_from_bech32_str(String::from(ACCOUNT).trim()).unwrap());

    let builder = [UTXO1, UTXO2].iter().fold(builder, |builder, key| {
        builder.add_key(SecretKey::try_from_bech32_str(String::from(*key).trim()).unwrap())
    });

    let mut free_keys = builder.build_free_utxos().unwrap();

    let account = builder.build_wallet().unwrap();

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    for fragment in state.initial_contents() {
        free_keys.check_fragment(&fragment.hash(), fragment);
    }

    assert_eq!(free_keys.unconfirmed_value(), Some(WALLET_VALUE));

    let (fragment, ignored) = dump_free_utxo(&settings, &address, &mut free_keys)
        .next()
        .unwrap();

    assert!(ignored.is_empty());

    state
        .apply_fragments(&[fragment.to_raw()])
        .expect("the dump fragments should be valid");
}

#[test]
fn cast_vote() {
    let builder = wallet::RecoveryBuilder::new();

    let builder = builder
        .account_secret_key(SecretKey::try_from_bech32_str(String::from(ACCOUNT).trim()).unwrap());

    let builder = [UTXO1, UTXO2].iter().fold(builder, |builder, key| {
        builder.add_key(SecretKey::try_from_bech32_str(String::from(*key).trim()).unwrap())
    });

    let mut free_keys = builder.build_free_utxos().unwrap();

    let mut account = builder.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    for fragment in state.initial_contents() {
        free_keys.check_fragment(&fragment.hash(), fragment);
    }

    assert_eq!(free_keys.unconfirmed_value(), Some(WALLET_VALUE));

    let (fragment, ignored) = dump_free_utxo(&settings, &address, &mut free_keys)
        .next()
        .unwrap();

    assert!(ignored.is_empty());

    state
        .apply_fragments(&[fragment.to_raw()])
        .expect("the dump fragments should be valid");

    account.confirm(&fragment.hash());

    let vote_plan_id: [u8; 32] = hex::decode(
        "9bc7103b8f391d409175c2fa52ee43b55a6ab0874b7fbd293dec02ffb062cc5a",
    )
    .unwrap()[..]
        .try_into()
        .unwrap();

    let index = 0;
    let choice = Choice::new(1);

    let payload = Payload::Public { choice };

    let cast = VoteCast::new(vote_plan_id.into(), index, payload);

    let mut builder = wallet::TransactionBuilder::new(&settings, cast);

    let value = builder.estimate_fee_with(1, 0);

    let account_tx_builder = account.new_transaction(value);
    let input = account_tx_builder.input();
    let witness_builder = account_tx_builder.witness_builder();

    builder.add_input(input, witness_builder);

    let tx = builder.finalize_tx(()).unwrap();

    let fragment = Fragment::VoteCast(tx);
    let raw = fragment.to_raw();
    let id = raw.id();

    account_tx_builder.add_fragment_id(id);

    state
        .apply_fragments(&[raw])
        .expect("couldn't apply votecast fragment");
}
