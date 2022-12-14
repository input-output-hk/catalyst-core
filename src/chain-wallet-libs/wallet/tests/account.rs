mod utils;

use self::utils::State;
use chain_crypto::SecretKey;
use chain_impl_mockchain::{
    account::SpendingCounter,
    accounting::account::SpendingCounterIncreasing,
    certificate::VoteCast,
    fragment::Fragment,
    value::Value,
    vote::{Choice, Payload},
};

const BLOCK0: &[u8] = include_bytes!("../../test-vectors/block0");
const ACCOUNT_KEY: &str = include_str!("../../test-vectors/free_keys/key1.prv");

#[test]
fn update_state_overrides_old() {
    let mut account = wallet::Wallet::new_from_key(
        SecretKey::from_binary(
            hex::decode(String::from(ACCOUNT_KEY).trim())
                .unwrap()
                .as_ref(),
        )
        .unwrap(),
    );

    assert_eq!(account.confirmed_value(), Value::zero());

    account
        .set_state(
            Value(110),
            (0..SpendingCounterIncreasing::LANES)
                .map(|lane| SpendingCounter::new(lane, 1).unwrap())
                .collect(),
        )
        .unwrap();

    assert_eq!(account.confirmed_value(), Value(110));
}

#[test]
fn cast_vote() {
    let mut account = wallet::Wallet::new_from_key(
        SecretKey::from_binary(
            hex::decode(String::from(ACCOUNT_KEY).trim())
                .unwrap()
                .as_ref(),
        )
        .unwrap(),
    );

    let mut state = State::new(BLOCK0);
    let settings = state.settings().expect("valid initial settings");

    for fragment in state.initial_contents() {
        account.check_fragment(&fragment.hash(), fragment).unwrap();
        account.confirm(&fragment.hash());
    }

    let vote_plan_id = &state.active_vote_plans()[0];

    let choice = Choice::new(1);

    let current_time =
        std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(settings.block0_date.0);

    for i in 0..16 {
        let payload = Payload::Public { choice };
        let cast = VoteCast::new(vote_plan_id.clone(), i, payload);

        let mut builder = wallet::TransactionBuilder::new(
            settings.clone(),
            cast.clone(),
            wallet::time::max_expiration_date(&settings, current_time).unwrap(),
        );

        let value = builder.estimate_fee_with(1, 0);

        let account_tx_builder = account.new_transaction(value, i % 8).unwrap();
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
}
