mod utils;

use self::utils::State;
use chain_impl_mockchain::{fragment::Fragment, value::Value};
use wallet::{transaction::dump_icarus_utxo, RecoveryBuilder};

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const MNEMONICS: &str =
    "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";

#[test]
fn dump_from_icarus() {
    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");

    let mut yoroi = wallet
        .build_yoroi()
        .expect("recover an Icarus/Yoroi wallet");

    let mut account = wallet.build_wallet().expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");
    let address = account.account_id().address(settings.discrimination());

    assert!(
        yoroi.check_fragments(state.initial_contents()),
        "failed to check fragments"
    );

    let (transaction, _ignored) =
        dump_icarus_utxo(&settings, &address, &yoroi).expect("expected only one transaction");

    let fragment = Fragment::Transaction(transaction.clone());

    assert!(account.check_fragment(&fragment.hash(), &fragment));

    state
        .apply_fragments(&[Fragment::Transaction(transaction).to_raw()])
        .expect("the dump fragments should be valid");

    assert_eq!(account.confirmed_value(), Value::zero());
    assert_ne!(account.unconfirmed_value().unwrap(), Value::zero());

    account.confirm(&fragment.hash());

    assert_ne!(account.confirmed_value(), Value::zero());
}

#[test]
fn update_state_overrides_old() {
    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");

    let mut account = wallet.build_wallet().expect("recover account");

    assert_eq!(account.confirmed_value(), Value::zero());

    account.update_state(Value(110), 0);

    assert_eq!(account.confirmed_value(), Value(110));
}
