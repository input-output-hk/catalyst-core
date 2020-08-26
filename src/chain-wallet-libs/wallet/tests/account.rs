mod utils;

use self::utils::State;
use chain_impl_mockchain::{
    certificate::VoteCast,
    fragment::Fragment,
    value::Value,
    vote::{Choice, Payload},
};
use std::convert::TryInto;
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

    let (fragment, _ignored) = dump_icarus_utxo(&settings, &address, &mut yoroi)
        .next()
        .expect("expected only one transaction");

    assert!(account.check_fragment(&fragment.hash(), &fragment));

    state
        .apply_fragments(&[fragment.to_raw()])
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

#[test]
fn cast_vote() {
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

    let (fragment, _ignored) = dump_icarus_utxo(&settings, &address, &mut yoroi)
        .next()
        .expect("expected only one transaction");

    assert!(account.check_fragment(&fragment.hash(), &fragment));

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
