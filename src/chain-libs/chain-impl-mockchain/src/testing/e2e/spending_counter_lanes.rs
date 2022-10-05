use crate::{
    block::BlockDate,
    fee::LinearFee,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, wallet},
        WitnessMode,
    },
};

#[test]
pub fn ledger_accepts_signature_from_all_lanes() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new().with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![wallet("Alice").with(1_000), wallet("Bob").with(1_000)])
        .build()
        .unwrap();

    let mut alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 0 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction_at_lane(0);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 1 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction_at_lane(1);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 2 })
        .transaction(&alice, &bob, &mut ledger, 10);
    ledger
        .apply_transaction(fragment, BlockDate::first())
        .unwrap();
    alice.confirm_transaction_at_lane(2);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 0 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction_at_lane(0);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 1 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction_at_lane(1);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 2 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());
    alice.confirm_transaction_at_lane(2);

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 2 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_ok());

    let fragment = controller
        .fragment_factory()
        .witness_mode(WitnessMode::Account { lane: 2 })
        .transaction(&alice, &bob, &mut ledger, 10);
    assert!(ledger
        .apply_transaction(fragment, BlockDate::first())
        .is_err());
}
