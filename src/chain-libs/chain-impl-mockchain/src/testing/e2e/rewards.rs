use crate::{
    config::RewardParams,
    fee::LinearFee,
    rewards::Ratio,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, stake_pool, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};

use std::num::{NonZeroU32, NonZeroU64};

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

    ledger.distribute_rewards().unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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
    ledger.distribute_rewards().unwrap();

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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
    let mut bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let fragment_factory = controller.fragment_factory();
    let fragment = fragment_factory.transaction(&mut bob, &clarice, &mut ledger, 100);
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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

    let mut ledger_verifier = LedgerStateVerifier::new(ledger.clone().into());
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
    let mut bob = controller.wallet("Bob").unwrap();
    let clarice = controller.wallet("Clarice").unwrap();

    let fragment_factory = controller.fragment_factory();
    let fragment = fragment_factory.transaction(&mut bob, &clarice, &mut ledger, 100);
    assert!(ledger.produce_block(&stake_pool, vec![fragment]).is_ok());

    LedgerStateVerifier::new(ledger.clone().into())
        .info("before rewards distribution")
        .total_value_is(&Value(4100));

    ledger.distribute_rewards().unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
        .info("after rewards distribution")
        .total_value_is(&Value(4100));
}
