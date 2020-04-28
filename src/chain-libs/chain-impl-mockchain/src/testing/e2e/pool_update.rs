use crate::{
    fee::LinearFee,
    stake::Stake,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, stake_pool, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};

#[test]
pub fn delegations_are_preserved_after_pool_update() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000).delegates_to("stake_pool"),
        ])
        .build()
        .unwrap();

    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().serial = 111u128;

    assert!(controller
        .update(
            &stake_pool,
            new_stake_pool.clone(),
            vec![&alice],
            &mut ledger
        )
        .is_ok());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("distribution is ok")
        .distribution()
        .unassigned_is(Stake::from_value(Value(997)))
        .and()
        .dangling_is(Stake::from_value(Value::zero()))
        .and()
        .pools_total_stake_is(Stake::from_value(Value(1000)));

    LedgerStateVerifier::new(ledger.into())
        .info("delegation is the same")
        .account(bob.as_account_data())
        .delegation()
        .is_fully_delegated_to(new_stake_pool.id());
}

#[test]
pub fn pool_update_after_pool_retirement() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![wallet("Alice")
            .with(1_000)
            .owns_and_delegates_to("stake_pool")])
        .build()
        .unwrap();

    let alice = controller.wallet("Alice").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    assert!(controller
        .retire(&[&alice], &stake_pool, &mut ledger)
        .is_ok());

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().serial = 111u128;

    assert!(controller
        .update(&stake_pool, new_stake_pool, vec![&alice], &mut ledger)
        .is_err());
}

#[test]
pub fn pool_update_wrong_signature_too_few() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![
            wallet("Alice")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
            wallet("Bob")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
            wallet("Clarice")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
        ])
        .with_stake_pools(vec![
            stake_pool("stake_pool").with_permissions_threshold(2u8)
        ])
        .build()
        .unwrap();

    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().serial = 111u128;

    assert!(controller
        .update(
            &stake_pool,
            new_stake_pool.clone(),
            vec![&alice],
            &mut ledger
        )
        .is_err());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("stake pool serial is not updated")
        .stake_pool(&new_stake_pool.id())
        .serial_eq(stake_pool.info().serial);

    assert!(controller
        .update(
            &stake_pool,
            new_stake_pool.clone(),
            vec![&alice, &bob],
            &mut ledger
        )
        .is_ok());

    LedgerStateVerifier::new(ledger.into())
        .info("stake pool serial is updated")
        .stake_pool(&new_stake_pool.id())
        .serial_eq(new_stake_pool.info().serial);
}

#[test]
pub fn pool_update_wrong_signature_incorrect_owner() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![
            wallet("Alice")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
            wallet("Bob").with(1_000).delegates_to("stake_pool"),
        ])
        .build()
        .unwrap();

    let bob = controller.wallet("Bob").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().serial = 111u128;

    assert!(controller
        .update(&stake_pool, new_stake_pool, vec![&bob], &mut ledger)
        .is_err());
}

#[test]
pub fn pool_update_changes_owner() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![
            wallet("Alice")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
            wallet("Bob").with(1_000),
        ])
        .build()
        .unwrap();

    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().owners = vec![bob.public_key()];

    assert!(controller
        .update(
            &stake_pool,
            new_stake_pool.clone(),
            vec![&alice],
            &mut ledger
        )
        .is_ok());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("stake pool owner is updated")
        .stake_pool(&new_stake_pool.id())
        .owners_eq(vec![bob.public_key()]);

    assert!(controller
        .retire(&[&bob], &new_stake_pool, &mut ledger)
        .is_ok());

    LedgerStateVerifier::new(ledger.into())
        .info("stake pool is retired")
        .stake_pools()
        .is_retired(&new_stake_pool);
}

#[test]
pub fn pool_update_revert_changes() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(ConfigBuilder::new(0).with_fee(LinearFee::new(1, 1, 1)))
        .with_initials(vec![
            wallet("Alice")
                .with(1_000)
                .owns_and_delegates_to("stake_pool"),
            wallet("Bob").with(1_000),
        ])
        .build()
        .unwrap();

    let mut alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    let mut new_stake_pool = stake_pool.clone();
    new_stake_pool.info_mut().owners = vec![bob.public_key()];

    assert!(controller
        .update(
            &stake_pool,
            new_stake_pool.clone(),
            vec![&alice],
            &mut ledger
        )
        .is_ok());
    alice.confirm_transaction();
    LedgerStateVerifier::new(ledger.clone().into())
        .info("stake pool owner is updated")
        .stake_pool(&new_stake_pool.id())
        .owners_eq(vec![bob.public_key()]);

    let mut reverted_stake_pool = new_stake_pool.clone();
    reverted_stake_pool.info_mut().owners = vec![alice.public_key()];

    assert!(controller
        .update(
            &new_stake_pool,
            reverted_stake_pool,
            vec![&bob],
            &mut ledger
        )
        .is_ok());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("stake pool owner is updated")
        .stake_pool(&new_stake_pool.id())
        .owners_eq(vec![alice.public_key()]);

    println!(
        "{:?}",
        controller.retire(&[&alice], &new_stake_pool, &mut ledger)
    );

    //assert!(controller.retire(&vec![&alice],&new_stake_pool,&mut ledger).is_ok());

    LedgerStateVerifier::new(ledger.into())
        .info("stake pool is retired")
        .stake_pools()
        .is_retired(&new_stake_pool);
}
