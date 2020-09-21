use crate::{
    certificate::PoolId,
    config::RewardParams,
    fee::LinearFee,
    rewards::Ratio,
    testing::{
        builders::StakePoolBuilder,
        ledger::{ConfigBuilder, TestLedger},
        scenario::{prepare_scenario, stake_pool, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};

use std::num::{NonZeroU32, NonZeroU64};

pub mod tax;

#[test]
pub fn rewards_no_block() {
    let (mut ledger, _) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(100))
                .with_treasury(Value(100)),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .build()
        .unwrap();

    assert_eq!(ledger.can_distribute_reward(), false);

    ledger.distribute_rewards().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("after empty rewards distribution")
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(100))
        .and()
        .has_remaining_rewards_equals_to(&Value(100));
}

#[test]
pub fn rewards_empty_pots() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(0))
                .with_treasury(Value(0)),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .with_stake_pools(vec![stake_pool("stake_pool")
            .with_reward_account(true)
            .tax_ratio(1, 1)])
        .build()
        .unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after empty rewards distribution");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value::zero())
        .and()
        .has_remaining_rewards_equals_to(&Value::zero());

    let reward_account = stake_pool.reward_account().unwrap();

    ledger_verifier
        .account(reward_account.clone())
        .does_not_exist();
}

#[test]
pub fn rewards_owners_split() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(100))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 10,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000).owns("stake_pool"),
            wallet("Clarice").with(1_000).owns("stake_pool"),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 1)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());
    assert!(ledger.can_distribute_reward());
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("tets after reward distribution splitted into 3 accounts");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value::zero())
        .and()
        .has_remaining_rewards_equals_to(&Value(91));

    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1003));
    ledger_verifier
        .account(bob.as_account_data())
        .has_value(&Value(1003));
    ledger_verifier
        .account(clarice.as_account_data())
        .has_value(&Value(1003));
}

#[test]
pub fn rewards_owners_uneven_split() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(100))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 20,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000).owns("stake_pool"),
            wallet("Clarice").with(1_000).owns("stake_pool"),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 1)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("tets after reward distribution splitted into 3 accounts");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value::zero())
        .and()
        .has_remaining_rewards_equals_to(&Value(81));

    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1007));
    ledger_verifier
        .account(bob.as_account_data())
        .has_value(&Value(1006));
    ledger_verifier
        .account(clarice.as_account_data())
        .has_value(&Value(1006));
}

#[test]
pub fn rewards_single_owner() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(100))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 10,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 1)])
        .build()
        .unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution to single owner");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(0))
        .and()
        .has_remaining_rewards_equals_to(&Value(91));

    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1_009));
}

#[test]
pub fn rewards_reward_account() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .with_stake_pools(vec![stake_pool("stake_pool")
            .with_reward_account(true)
            .tax_ratio(1, 10)])
        .build()
        .unwrap();
    let stake_pool = controller.stake_pool("stake_pool").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution to reward account");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(90))
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    let reward_account = stake_pool.reward_account().unwrap();

    ledger_verifier
        .account(reward_account.clone())
        .has_value(&Value(9))
        .and()
        .has_last_reward(&Value(9));
}

#[test]
pub fn rewards_goes_to_treasury_if_stake_pool_is_retired() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .with_stake_pools(vec![stake_pool("stake_pool")
            .with_reward_account(true)
            .tax_ratio(1, 10)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());

    controller
        .retire(&[&alice], &stake_pool, &mut ledger)
        .unwrap();
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution to retired stake pool");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(99))
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    let reward_account = stake_pool.reward_account().unwrap();

    ledger_verifier
        .account(reward_account.clone())
        .does_not_exist();
}

#[test]
pub fn rewards_from_fees() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000),
            wallet("Clarice").with(1_000),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool")
            .with_reward_account(true)
            .tax_ratio(1, 10)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let fragment_factory = controller.fragment_factory();
    let fragment = fragment_factory.transaction(&bob, &clarice, &mut ledger, 100);
    assert!(ledger.produce_block(&stake_pool, vec![fragment]).is_ok());

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
    ledger_verifier.info("before rewards distribution with single transaction");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value(3))
        .and()
        .has_treasury_equals_to(&Value::zero())
        .and()
        .has_remaining_rewards_equals_to(&Value(1000));

    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution with single transaction");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(92))
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    let reward_account = stake_pool.reward_account().unwrap();

    ledger_verifier
        .account(reward_account.clone())
        .has_value(&Value(10));
}

#[test]
pub fn rewards_stake_pool_with_delegation() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000).delegates_to("stake_pool"),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 2)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();

    assert!(ledger.produce_empty_block(&stake_pool).is_ok());

    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution with delegation");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value::zero())
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1049));
    ledger_verifier
        .account(bob.as_account_data())
        .has_value(&Value(1050));
}

#[test]
pub fn rewards_total_amount_is_constant_after_reward_distribution() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(1000))
                .with_treasury(Value(100))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000).delegates_to("stake_pool"),
            wallet("Clarice").with(1_000),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 2)])
        .build()
        .unwrap();

    let stake_pool = controller.stake_pool("stake_pool").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let fragment_factory = controller.fragment_factory();
    let fragment = fragment_factory.transaction(&bob, &clarice, &mut ledger, 100);
    assert!(ledger.produce_block(&stake_pool, vec![fragment]).is_ok());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("before rewards distribution")
        .total_value_is(&Value(4100));

    ledger.distribute_rewards().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("after rewards distribution")
        .total_value_is(&Value(4100));
}

#[test]
pub fn rewards_are_propotional_to_stake_pool_effectivness_in_building_blocks() {
    let slots_per_epoch = 100;
    let reward_constant = 100;
    let ratio_numerator = 1;
    let expected_total_reward = Value(reward_constant - ratio_numerator);

    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_slots_per_epoch(slots_per_epoch)
                .with_rewards(Value(1_000_000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: reward_constant,
                    ratio: Ratio {
                        numerator: ratio_numerator,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice")
                .with(1_000_000)
                .owns_and_delegates_to("alice_stake_pool"),
            wallet("Bob")
                .with(1_000_000)
                .owns_and_delegates_to("bob_stake_pool"),
            wallet("Clarice")
                .with(1_000_000)
                .owns_and_delegates_to("clarice_stake_pool"),
            wallet("Carol").with(1_000_000),
            wallet("David").with(1_000_000),
        ])
        .with_stake_pools(vec![
            stake_pool("alice_stake_pool").tax_ratio(1, 2),
            stake_pool("bob_stake_pool").tax_ratio(1, 2),
            stake_pool("clarice_stake_pool").tax_ratio(1, 2),
        ])
        .build()
        .unwrap();

    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let alice_stake_pool = controller.stake_pool("alice_stake_pool").unwrap();
    let bob_stake_pool = controller.stake_pool("bob_stake_pool").unwrap();
    let clarice_stake_pool = controller.stake_pool("clarice_stake_pool").unwrap();

    let mut carol = controller.wallet("Carol").unwrap();
    let david = controller.wallet("David").unwrap();

    let fragment_factory = controller.fragment_factory();

    while ledger.date().slot_id < 99 {
        let fragment = fragment_factory.transaction(&carol, &david, &mut ledger, 100);
        let block_was_created = ledger
            .fire_leadership_event(controller.initial_stake_pools(), vec![fragment])
            .unwrap();
        if block_was_created {
            carol.confirm_transaction();
        }
    }

    let expected_alice_reward =
        (calculate_reward(expected_total_reward, &alice_stake_pool.id(), &ledger) + alice.value())
            .unwrap();
    let expected_bob_reward =
        (calculate_reward(expected_total_reward, &bob_stake_pool.id(), &ledger) + bob.value())
            .unwrap();
    let expected_clarice_reward =
        (calculate_reward(expected_total_reward, &clarice_stake_pool.id(), &ledger)
            + clarice.value())
        .unwrap();

    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier
        .info("after rewards distribution for alice")
        .account(alice.as_account_data())
        .has_value(&expected_alice_reward);
    ledger_verifier
        .info("after rewards distribution for bob")
        .account(bob.as_account_data())
        .has_value(&expected_bob_reward);
    ledger_verifier
        .info("after rewards distribution for clarice")
        .account(clarice.as_account_data())
        .has_value(&expected_clarice_reward);
}

fn calculate_reward(expected_total_reward: Value, pool_id: &PoolId, ledger: &TestLedger) -> Value {
    let reward_unit = expected_total_reward.split_in(ledger.leaders_log().total());
    let block_count = ledger.leaders_log_for(pool_id);
    reward_unit.parts.scale(block_count).unwrap()
}

#[test]
pub fn rewards_owner_of_many_stake_pool() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("stake_pool"),
            wallet("Bob").with(1_000),
            wallet("Clarice").with(1_000),
        ])
        .with_stake_pools(vec![stake_pool("stake_pool").tax_ratio(1, 10)])
        .build()
        .unwrap();

    let first_alice_stake_pool = controller.stake_pool("stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();
    let bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let total_ada_before = ledger.total_funds();

    let second_alice_stake_pool = StakePoolBuilder::new()
        .with_owners(vec![alice.public_key()])
        .with_ratio_tax_type(1, 10, None)
        .build();

    controller
        .register(&alice, &second_alice_stake_pool, &mut ledger)
        .unwrap();

    // produce 2 blocks for each stake pool
    let fragment_factory = controller.fragment_factory();

    let fragment = fragment_factory.transaction(&bob, &clarice, &mut ledger, 100);
    assert!(ledger
        .produce_block(&first_alice_stake_pool, vec![fragment])
        .is_ok());

    let fragment = fragment_factory.transaction(&clarice, &bob, &mut ledger, 100);
    assert!(ledger
        .produce_block(&second_alice_stake_pool, vec![fragment])
        .is_ok());

    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution with two transactions");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(98))
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    ledger_verifier.total_value_is(&total_ada_before);

    // check owner account (10 from rewards - 3 from register stake pool fee)
    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1007));
}

#[test]
pub fn rewards_delegators_of_many_stake_pool() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_fee(LinearFee::new(1, 1, 1))
                .with_rewards(Value(1000))
                .with_treasury(Value(0))
                .with_rewards_params(RewardParams::Linear {
                    constant: 100,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("alice_stake_pool"),
            wallet("Bob").with(1_000).owns("bob_stake_pool"),
            wallet("Clarice").with(1_000),
            wallet("David").with(1_000),
            wallet("Eve").with(1_000),
        ])
        .with_stake_pools(vec![
            stake_pool("alice_stake_pool").tax_ratio(1, 10),
            stake_pool("bob_stake_pool").tax_ratio(1, 10),
        ])
        .build()
        .unwrap();

    let alice_stake_pool = controller.stake_pool("alice_stake_pool").unwrap();
    let bob_stake_pool = controller.stake_pool("bob_stake_pool").unwrap();

    let clarice = controller.wallet("Clarice").unwrap();
    let david = controller.wallet("David").unwrap();
    let eve = controller.wallet("Eve").unwrap();

    let total_ada_before = ledger.total_funds();

    controller
        .delegates_to_many(
            &eve,
            &[(&alice_stake_pool, 1), (&bob_stake_pool, 1)],
            &mut ledger,
        )
        .unwrap();

    // produce 2 blocks for each stake pool
    let fragment_factory = controller.fragment_factory();

    let fragment = fragment_factory.transaction(&david, &clarice, &mut ledger, 100);
    assert!(ledger
        .produce_block(&alice_stake_pool, vec![fragment])
        .is_ok());

    let fragment = fragment_factory.transaction(&clarice, &david, &mut ledger, 100);
    assert!(ledger
        .produce_block(&bob_stake_pool, vec![fragment])
        .is_ok());

    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.into());
    ledger_verifier.info("after rewards distribution with two transactions");

    ledger_verifier
        .pots()
        .has_fee_equals_to(&Value::zero())
        .and()
        .has_treasury_equals_to(&Value(2))
        .and()
        .has_remaining_rewards_equals_to(&Value(901));

    ledger_verifier.total_value_is(&total_ada_before);

    // check owner account (94 from rewards - 3 from register stake pool fee)
    ledger_verifier
        .account(eve.as_account_data())
        .has_value(&Value(1093));
}
