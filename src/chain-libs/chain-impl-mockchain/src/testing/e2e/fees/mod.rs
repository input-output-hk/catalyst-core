use crate::{
    fee::{LinearFee, PerCertificateFee},
    testing::{
        builders::StakePoolBuilder,
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};
use chain_addr::Discrimination;

use std::num::NonZeroU64;

#[test]
pub fn per_certificate_fees() {
    let input_constant = 1;
    let input_coefficient = 1;
    let default_certificate_fee = 10;
    let certificate_pool_registration_fee = 1;
    let certificate_stake_delegation = 20;
    let certificate_owner_stake_delegation = 300;

    let mut fee_amount = 0;
    let mut alice_funds = 1_000;
    let mut bob_funds = 1_000;
    let transaction_fee = input_coefficient + input_coefficient;
    let expected_pool_registration_fee = transaction_fee + certificate_pool_registration_fee;
    let expected_owner_delegation_fee = transaction_fee + certificate_owner_stake_delegation;

    let expected_delegation_fee = transaction_fee + certificate_stake_delegation;
    let expected_retirement_fee = transaction_fee + default_certificate_fee;

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_discrimination(Discrimination::Test)
                .with_fee(LinearFee::new(
                    input_constant,
                    input_coefficient,
                    default_certificate_fee,
                ))
                .with_per_certificate_fee(PerCertificateFee::new(
                    NonZeroU64::new(certificate_pool_registration_fee),
                    NonZeroU64::new(certificate_stake_delegation),
                    NonZeroU64::new(certificate_owner_stake_delegation),
                )),
        )
        .with_initials(vec![
            wallet("Alice").with(alice_funds),
            wallet("Bob").with(bob_funds),
        ])
        .build()
        .unwrap();

    let mut alice = controller.wallet("Alice").unwrap();
    let mut bob = controller.wallet("Bob").unwrap();
    let stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .build();

    //1. register stake pool
    controller
        .register(&alice, &stake_pool, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    fee_amount += expected_pool_registration_fee;
    alice_funds -= expected_pool_registration_fee;

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
    ledger_verifier
        .info("after register")
        .pots()
        .has_fee_equals_to(&Value(fee_amount));
    ledger_verifier
        .info("after register")
        .account(alice.as_account_data())
        .has_value(&Value(alice_funds));

    //2. owner delegate to stake pool
    controller
        .owner_delegates(&alice, &stake_pool, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    fee_amount += expected_owner_delegation_fee;
    alice_funds -= expected_owner_delegation_fee;

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());

    ledger_verifier
        .info("after owner delegation")
        .pots()
        .has_fee_equals_to(&Value(fee_amount));

    ledger_verifier
        .info("after owner delegation")
        .account(alice.as_account_data())
        .has_value(&Value(alice_funds));

    //3. delegate to stake pool
    controller
        .delegates(&bob, &stake_pool, &mut ledger)
        .unwrap();
    bob.confirm_transaction();

    fee_amount += expected_delegation_fee;
    bob_funds -= expected_delegation_fee;

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
    ledger_verifier
        .info("after delegation")
        .pots()
        .has_fee_equals_to(&Value(fee_amount));
    ledger_verifier
        .info("after delegation")
        .account(bob.as_account_data())
        .has_value(&Value(bob_funds));

    //4. retire stake pool
    controller
        .retire(&[&alice], &stake_pool, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    fee_amount += expected_retirement_fee;
    alice_funds -= expected_retirement_fee;

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier
        .info("after retire")
        .pots()
        .has_fee_equals_to(&Value(fee_amount));
    ledger_verifier
        .info("after retire")
        .account(alice.as_account_data())
        .has_value(&Value(alice_funds));
}

#[test]
pub fn owner_delegates_fee() {
    let reward_value = 1_000_000;
    let treasury_value = 1_000_000;
    let alice_funds = 1_000;

    let expected_total_funds_before = reward_value + treasury_value + alice_funds;

    // constant (1) + coefficient (1) * inputs (1) + certificate_fee (1) = 3;
    let expected_fee_amount = 1 + 1 + 1;
    let expected_total_funds_after = expected_total_funds_before;

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(reward_value))
                .with_treasury(Value(treasury_value))
                .with_discrimination(Discrimination::Test)
                .with_fee(LinearFee::new(1, 1, 1)),
        )
        .with_initials(vec![wallet("Alice").with(alice_funds).owns("stake_pool")])
        .build()
        .unwrap();

    let mut alice = controller.wallet("Alice").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
        .total_value_is(&Value(expected_total_funds_before));

    controller
        .owner_delegates(&alice, &stake_pool, &mut ledger)
        .unwrap();
    alice.confirm_transaction();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());

    ledger_verifier
        .info("after owner_delegates")
        .pots()
        .has_fee_equals_to(&Value(expected_fee_amount));

    ledger_verifier.total_value_is(&Value(expected_total_funds_after));
}
