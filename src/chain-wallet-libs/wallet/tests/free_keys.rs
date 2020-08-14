mod utils;

use self::utils::State;
use chain_crypto::{bech32::Bech32, SecretKey};
use chain_impl_mockchain::value::Value;
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
