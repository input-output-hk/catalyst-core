mod utils;

use self::utils::State;
use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    certificate::VoteCast,
    fragment::Fragment,
    value::Value,
    vote::{Choice, Payload},
};
use std::convert::TryInto;
use wallet::RecoveryBuilder;

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const ACCOUNT_KEY: &str = include_str!("../../test-vectors/free_keys/key1.prv");
const MNEMONICS: &str =
    "neck bulb teach illegal soul cry monitor claw amount boring provide village rival draft stone";

#[test]
fn update_state_overrides_old() {
    let wallet = RecoveryBuilder::new()
        .mnemonics(&bip39::dictionary::ENGLISH, MNEMONICS)
        .expect("valid mnemonics");

    let mut account = wallet.build_wallet().expect("recover account");

    assert_eq!(account.confirmed_value(), Value::zero());

    account.update_state(Value(110), 0.into());

    assert_eq!(account.confirmed_value(), Value(110));
}

#[test]
fn cast_vote() {
    let mut account = wallet::RecoveryBuilder::new()
        .account_secret_key(
            SecretKey::from_binary(
                hex::decode(String::from(ACCOUNT_KEY).trim())
                    .unwrap()
                    .as_ref(),
            )
            .unwrap(),
        )
        .build_wallet()
        .expect("recover account");

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");

    for fragment in state.initial_contents() {
        account.check_fragment(&fragment.hash(), fragment);
        account.confirm(&fragment.hash());
    }

    let vote_plan_id: [u8; 32] = hex::decode(
        "784d95bf9090969df0398f94c48baffbba8ea9f6e7a1e7d808a156330fdf33e1",
    )
    .unwrap()[..]
        .try_into()
        .unwrap();

    let index = 0;
    let choice = Choice::new(1);

    let payload = Payload::Public { choice };

    let cast = VoteCast::new(vote_plan_id.into(), index, payload);

    let current_time =
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(settings.block0_date.0);

    let mut builder = wallet::TransactionBuilder::new(
        &settings,
        cast,
        wallet::time::max_expiration_date(&settings, current_time).unwrap(),
    );

    let value = builder.estimate_fee_with(1, 0);

    let account_tx_builder = account.new_transaction(value).unwrap();
    let input = account_tx_builder.input();
    let witness_builder = account_tx_builder.witness_builder();

    builder.add_input(input, witness_builder);

    let tx = builder.finalize_tx(()).unwrap();

    let fragment = Fragment::VoteCast(tx);
    let id = fragment.hash();

    account_tx_builder.add_fragment_id(id);

    state
        .apply_fragments(&[fragment])
        .expect("couldn't apply votecast fragment");
}
