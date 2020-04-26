use crate::{
    config::RewardParams,
    fee::LinearFee,
    rewards::Ratio,
    stake::Stake,
    testing::{
        ledger::ConfigBuilder,
        scenario::{prepare_scenario, stake_pool, wallet},
        verifiers::LedgerStateVerifier,
    },
    value::Value,
};
use chain_addr::Discrimination;
use std::num::{NonZeroU32, NonZeroU64};

#[test]
pub fn stake_distribution_to_many_stake_pools() {
    let (mut ledger, controller) = prepare_scenario()
        .with_config(
            ConfigBuilder::new(0)
                .with_discrimination(Discrimination::Test)
                .with_fee(LinearFee::new(1, 1, 1)),
        )
        .with_initials(vec![
            wallet("Alice").with(1_000).owns("alice_stake_pool"),
            wallet("Bob").with(1_000).owns("bob_stake_pool"),
            wallet("Clarice").with(1_000).owns("clarice_stake_pool"),
            wallet("David").with(1_003),
        ])
        .build()
        .unwrap();

    let alice_stake_pool = controller.stake_pool("alice_stake_pool").unwrap();
    let bob_stake_pool = controller.stake_pool("bob_stake_pool").unwrap();
    let clarice_stake_pool = controller.stake_pool("clarice_stake_pool").unwrap();

    let david = controller.wallet("David").unwrap();

    let delegation_ratio = vec![
        (&alice_stake_pool, 2u8),
        (&bob_stake_pool, 3u8),
        (&clarice_stake_pool, 5u8),
    ];

    controller
        .delegates_to_many(&david, &delegation_ratio, &mut ledger)
        .unwrap();

    let expected_distribution = vec![
        (alice_stake_pool.id(), Value(200)),
        (bob_stake_pool.id(), Value(300)),
        (clarice_stake_pool.id(), Value(500)),
    ];

    LedgerStateVerifier::new(ledger.into())
        .info("after delegation to many stake pools")
        .distribution()
        .pools_distribution_is(expected_distribution);
}

#[test]
pub fn stake_distribution_changes_after_rewards_are_collected() {
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
            wallet("Alice").with(1_000).owns("alice_stake_pool"),
            wallet("Bob").with(1_000),
            wallet("Clarice").with(1_000),
        ])
        .with_stake_pools(vec![stake_pool("alice_stake_pool").tax_ratio(1, 1)])
        .build()
        .unwrap();

    let alice_stake_pool = controller.stake_pool("alice_stake_pool").unwrap();
    let alice = controller.wallet("Alice").unwrap();

    controller
        .owner_delegates(&alice, &alice_stake_pool, &mut ledger)
        .unwrap();

    LedgerStateVerifier::new(ledger.clone().into())
        .info("before rewards collection")
        .distribution()
        .unassigned_is(Stake::from_value(Value(2000)))
        .pools_distribution_is(vec![(alice_stake_pool.id(), Value(1000))]);

    assert!(ledger.produce_empty_block(&alice_stake_pool).is_ok());
    ledger.distribute_rewards().unwrap();

    LedgerStateVerifier::new(ledger.into())
        .info("after rewards collection")
        .distribution()
        .unassigned_is(Stake::from_value(Value(2000)))
        .pools_distribution_is(vec![(alice_stake_pool.id(), Value(1009))]);
}
