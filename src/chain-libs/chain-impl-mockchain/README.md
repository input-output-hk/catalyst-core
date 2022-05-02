# Mock implementation of the chain.

It defines mock implementation of the chain and can be used
for testing blockchain algorithms and work as an example of
the implementation.

# Testing

Tests are covering internals of ledger library, validating particular modules.
There are also scenarios for testing ledger in a 'turn-based' aproach. For example:

```
    /// Prepare blockchain settings and actors
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

    /// Retrieve actors
    let alice_stake_pool = controller.stake_pool("alice_stake_pool").unwrap();
    let bob_stake_pool = controller.stake_pool("bob_stake_pool").unwrap();
    let clarice_stake_pool = controller.stake_pool("clarice_stake_pool").unwrap();

    let david = controller.wallet("David").unwrap();

    // prepare delegation ratio
    let delegation_ratio = vec![
        (&alice_stake_pool, 2u8),
        (&bob_stake_pool, 3u8),
        (&clarice_stake_pool, 5u8),
    ];

    /// post delegation certificates
    controller
        .delegates_to_many(&david, &delegation_ratio, &mut ledger)
        .unwrap();

    /// verify distribution is correct
    let expected_distribution = vec![
        (alice_stake_pool.id(), Value(200)),
        (bob_stake_pool.id(), Value(300)),
        (clarice_stake_pool.id(), Value(500)),
    ];

    LedgerStateVerifier::new(ledger.clone().into())
        .info("after delegation to many stake pools")
        .distribution()
        .pools_distribution_is(expected_distribution);

```


### How to run tests
```
cd chain-impl-mockchain
cargo test
```
