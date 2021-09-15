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
use quickcheck_macros::quickcheck;

use std::num::NonZeroU64;

const ALICE: &str = "Alice";
const BOB: &str = "Bob";
const STAKE_POOL: &str = "stake_pool";

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
            ConfigBuilder::new()
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
            wallet(ALICE).with(alice_funds),
            wallet(BOB).with(bob_funds),
        ])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let mut bob = controller.wallet(BOB).unwrap();
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
        .retire(Some(&alice), &stake_pool, &mut ledger)
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
            ConfigBuilder::new()
                .with_rewards(Value(reward_value))
                .with_treasury(Value(treasury_value))
                .with_discrimination(Discrimination::Test)
                .with_fee(LinearFee::new(1, 1, 1)),
        )
        .with_initials(vec![wallet(ALICE).with(alice_funds).owns(STAKE_POOL)])
        .build()
        .unwrap();

    let mut alice = controller.wallet(ALICE).unwrap();
    let stake_pool = controller.stake_pool(STAKE_POOL).unwrap();

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

#[test]
/// Verifies that after a transaction in a ledger without fees, the total funds do not change and
/// the fee pots remain empty.
fn transaction_without_fees() {
    verify_total_funds_after_transaction_with_fee(0);
}

/// Verifies that after a transaction in a ledger with fees, the total funds do not change and the
/// fee pots contain the fee.
#[quickcheck]
fn transaction_with_fees(fee: u64) {
    verify_total_funds_after_transaction_with_fee(fee);
}

fn verify_total_funds_after_transaction_with_fee(fee: u64) {
    const BOB_FUNDS: u64 = 42;
    let transfer = fee + 13; // The transfer should be large enough to cover the fee
    let alice_funds = transfer + 13; // Alice should have enough funds to cover the transfer

    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new().with_fee(LinearFee::new(fee, 0, 0)))
        .with_initials(vec![
            wallet(ALICE).with(alice_funds),
            wallet(BOB).with(BOB_FUNDS),
        ])
        .build()
        .expect("Could not build scenario");

    let total_funds = ledger.total_funds();

    let mut alice_wallet = controller.wallet(ALICE).unwrap();
    let bob_wallet = controller.wallet(BOB).unwrap();

    controller
        .transfer_funds(&alice_wallet, &bob_wallet, &mut ledger, transfer)
        .unwrap();
    alice_wallet.confirm_transaction();

    LedgerStateVerifier::new(ledger.clone().into())
        .address_has_expected_balance(
            alice_wallet.as_account_data(),
            Value(alice_funds - transfer),
        )
        .address_has_expected_balance(
            bob_wallet.as_account_data(),
            Value(BOB_FUNDS + transfer - fee),
        )
        .total_value_is(&total_funds)
        .pots()
        .has_fee_equals_to(&Value(fee));
}
