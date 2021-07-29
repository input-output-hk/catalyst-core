mod utils;

use self::utils::State;
use chain_crypto::SecretKey;
use chain_impl_mockchain::value::Value;
use wallet::transaction::dump_free_utxo;

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const ACCOUNT: &str = include_str!("../../test-vectors/free_keys/key1.prv");
const UTXO1: &str = include_str!("../../test-vectors/free_keys/key2.prv");
const UTXO2: &str = include_str!("../../test-vectors/free_keys/key3.prv");
const WALLET_VALUE: Value = Value(10_000 + 1000);

#[test]
fn test_free_utxo_key_dump() {
    let builder = wallet::RecoveryBuilder::new();

    let builder = builder.account_secret_key(
        SecretKey::from_binary(hex::decode(String::from(ACCOUNT).trim()).unwrap().as_ref())
            .unwrap(),
    );

    let builder = [UTXO1, UTXO2].iter().fold(builder, |builder, key| {
        builder.add_key(
            SecretKey::from_binary(hex::decode(String::from(*key).trim()).unwrap().as_ref())
                .unwrap(),
        )
    });

    let mut free_keys = builder.build_free_utxos().unwrap();

    let mut account = builder.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    for fragment in state.initial_contents() {
        account.check_fragment(&fragment.hash(), fragment);
        free_keys.check_fragment(&fragment.hash(), fragment);

        account.confirm(&fragment.hash());
        free_keys.confirm(&fragment.hash());
    }

    assert_eq!(free_keys.confirmed_value(), WALLET_VALUE);

    let (fragment, ignored) = dump_free_utxo(&settings, &address, &mut free_keys)
        .next()
        .unwrap();

    assert!(ignored.is_empty());

    account.check_fragment(&fragment.hash(), &fragment);

    state
        .apply_fragments(&[fragment.to_raw()])
        .expect("the dump fragments should be valid");

    let (_counter, value) = state.get_account_state(account.account_id()).unwrap();

    account.confirm(&fragment.hash());

    assert_eq!(account.confirmed_value(), value);
}
