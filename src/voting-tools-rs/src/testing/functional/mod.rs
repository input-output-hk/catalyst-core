use crate::test_api::{MockDbProvider, VerifiableSnapshotOutput};
use crate::Delegations;
use mainnet_lib::wallet_state::MainnetWalletStateBuilder;
use mainnet_lib::{CardanoWallet, MainnetNetworkBuilder};

#[test]
fn cip15_correctly_signed_before_snapshot() {
    let stake = 10_000;
    let alice_wallet = CardanoWallet::new(stake);

    let (db_sync, _node, _) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_direct_voter())
        .build();

    let db = MockDbProvider::from(db_sync);
    let outputs = crate::voting_power(&db, None, None, None).unwrap();

    assert_eq!(outputs.len(), 1);

    let assertions = outputs[0].assert();

    assertions.voting_power(stake);
    assertions.delegations(&VotingPowerSource::Legacy(
        alice_wallet.catalyst_public_key().to_hex(),
    ));
    assertions.reward_address(&alice_wallet.reward_address());
    assertions.stake_key(&alice_wallet.stake_public_key());
}

#[test]
fn cip36_correctly_signed_before_snapshot() {
    let stake = 10_000;
    let alice_wallet = CardanoWallet::new(stake);
    let bob_wallet = CardanoWallet::new(stake);
    let clarice_wallet = CardanoWallet::new(stake);

    let (db_sync, _node, _) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_representative())
        .with(bob_wallet.as_representative())
        .with(clarice_wallet.as_delegator(vec![(&alice_wallet, 1), (&bob_wallet, 1)]))
        .build();

    let db = MockDbProvider::from(db_sync);
    let outputs = crate::voting_power(&db, None, None, None).unwrap();

    assert_eq!(outputs.len(), 1);

    let assertions = outputs[0].assert();

    assertions.voting_power(stake);
    assertions.delegations(&VotingPowerSource::Delegated(vec![
        (alice_wallet.catalyst_public_key().to_hex(), 1),
        (bob_wallet.catalyst_public_key().to_hex(), 1),
    ]));
    assertions.reward_address(&clarice_wallet.reward_address());
    assertions.stake_key(&clarice_wallet.stake_public_key());
}
