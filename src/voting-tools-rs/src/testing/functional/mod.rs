use crate::test_api::{MockDbProvider, VerifiableSnapshotOutput};
use assert_fs::TempDir;
use mainnet_lib::{MainnetNetworkBuilder, MainnetWallet, MainnetWalletStateBuilder};

#[test]
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
    assertions.reward_address(&alice_wallet.reward_address());
    assertions.stake_key(&alice_wallet.stake_public_key());
}
