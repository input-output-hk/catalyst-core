use crate::test_api::{MockDbProvider, VerifiableSnapshotOutput};
use crate::Delegations;
use assert_fs::TempDir;
use mainnet_lib::{MainnetNetworkBuilder, MainnetWallet, MainnetWalletStateBuilder};

#[test]
#[ignore]
fn cip15_correctly_signed_before_snapshot() {
    let temp_dir = TempDir::new().unwrap();

    let stake = 10_000;
    let alice_wallet = MainnetWallet::new(stake);

    let (db_sync, _) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_direct_voter())
        .build(&temp_dir);

    let db = MockDbProvider::from(db_sync);
    let outputs = crate::voting_power(&db, None, None, None).unwrap();

    assert_eq!(outputs.len(), 1);

    let assertions = outputs[0].assert();

    assertions.voting_power(stake);
    assertions.delegations(&Delegations::Legacy(
        alice_wallet.stake_public_key().to_hex(),
    ));
    assertions.reward_address(&alice_wallet.reward_address());
    assertions.stake_key(&alice_wallet.stake_public_key());
}

#[test]
fn cip36_correctly_signed_before_snapshot() {
    let temp_dir = TempDir::new().unwrap();

    let stake = 10_000;
    let alice_wallet = MainnetWallet::new(stake);
    let bob_wallet = MainnetWallet::new(stake);
    let clarice_wallet = MainnetWallet::new(stake);

    let (db_sync, _) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_representative())
        .with(bob_wallet.as_representative())
        .with(clarice_wallet.as_delegator(vec![(&alice_wallet, 1), (&bob_wallet, 1)]))
        .build(&temp_dir);

    let db = MockDbProvider::from(db_sync);
    let outputs = crate::voting_power(&db, None, None, None).unwrap();

    assert_eq!(outputs.len(), 1);

    let assertions = outputs[0].assert();

    assertions.voting_power(stake);
    assertions.delegations(&Delegations::Delegated(vec![
        (alice_wallet.catalyst_public_key().to_hex(), 1),
        (bob_wallet.catalyst_public_key().to_hex(), 1),
    ]));
    assertions.reward_address(&clarice_wallet.reward_address());
    assertions.stake_key(&clarice_wallet.stake_public_key());
}
