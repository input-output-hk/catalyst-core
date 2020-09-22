use crate::{
    config::RewardParams,
    rewards::Ratio,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, stake_pool, template::StakePoolDefBuilder, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};

use std::num::{NonZeroU32, NonZeroU64};

#[test]
pub fn tax_ratio_limit() {
    let total_reward = 100;
    let expected_stake_pool_reward = 30;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").tax_ratio(1, 2).tax_limit(30),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn fixed_tax_ok() {
    let total_reward = 100;
    let expected_stake_pool_reward = 10;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").fixed_tax(10),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn fixed_tax_too_high() {
    let total_reward = 100;
    let expected_stake_pool_reward = 100;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").fixed_tax(1_000),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn fixed_tax_limit() {
    let total_reward = 100;
    let expected_stake_pool_reward = 10;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").fixed_tax(10).tax_limit(5),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn fixed_tax_lower_than_limit() {
    let total_reward = 100;
    let expected_stake_pool_reward = 20;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").fixed_tax(20).tax_limit(30),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn tax_ratio_lower_than_limit() {
    let total_reward = 100;
    let expected_stake_pool_reward = 50;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").tax_ratio(1, 2).tax_limit(60),
        expected_stake_pool_reward,
    );
}

#[test]
pub fn no_tax() {
    let total_reward = 100;
    let expected_stake_pool_reward = 0;
    verify_distribute_rewards(
        total_reward,
        stake_pool("stake_pool").no_tax(),
        expected_stake_pool_reward,
    );
}

fn verify_distribute_rewards(
    total_reward: u64,
    stake_pool_builder: &mut StakePoolDefBuilder,
    expected_stake_pool_reward: u64,
) {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_rewards(Value(total_reward + 1))
                .with_treasury(Value::zero())
                .with_rewards_params(RewardParams::Linear {
                    constant: total_reward + 1,
                    ratio: Ratio {
                        numerator: 1,
                        denominator: NonZeroU64::new(1).unwrap(),
                    },
                    epoch_start: 0,
                    epoch_rate: NonZeroU32::new(1).unwrap(),
                }),
        )
        .with_initials(vec![wallet("Alice").with(1_000).owns("stake_pool")])
        .with_stake_pools(vec![stake_pool_builder])
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
        .has_treasury_equals_to(&Value(total_reward - expected_stake_pool_reward))
        .and()
        .has_remaining_rewards_equals_to(&Value(1));

    ledger_verifier
        .account(alice.as_account_data())
        .has_value(&Value(1_000 + expected_stake_pool_reward));
}
