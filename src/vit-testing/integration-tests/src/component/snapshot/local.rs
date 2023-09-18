/*
use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::CardanoWallet;
use assert_fs::TempDir;
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_lib::VoterHIR;
use snapshot_trigger_service::config::JobParameters;
use vitup::config::{DIRECT_VOTING_GROUP, REP_VOTING_GROUP};
*/
/* BROKEN TEST - Because VoterHIR has no `eq`
#[test]
pub fn mixed_registration_transactions() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;

    let alice = CardanoWallet::new(stake);
    let bob = CardanoWallet::new(stake);
    let clarice = CardanoWallet::new(stake);

    let david = CardanoWallet::new(500);
    let edgar = CardanoWallet::new(1_000);
    let fred = CardanoWallet::new(8_000);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_delegator(vec![(&david, 1)]))
        .with(clarice.as_delegator(vec![(&david, 1), (&edgar, 1)]))
        .with(david.as_representative())
        .with(edgar.as_representative())
        .with(fred.as_representative())
        .build();

    let voters_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter_default(&reps)
        .to_voters_hirs();

    assert!(voters_hir
        .iter()
        .any(|hir| *hir == alice.as_voter_hir(DIRECT_VOTING_GROUP)));
    assert!(!voters_hir
        .iter()
        .any(|hir| *hir == bob.as_voter_hir(DIRECT_VOTING_GROUP)));
    assert!(voters_hir.iter().any(|hir| *hir
        == VoterHIR {
            voting_key: david.catalyst_public_key(),
            voting_group: REP_VOTING_GROUP.to_string(),
            voting_power: 15000u64.into(),
        }));
    assert!(voters_hir.iter().any(|hir| *hir
        == VoterHIR {
            voting_key: edgar.catalyst_public_key(),
            voting_group: REP_VOTING_GROUP.to_string(),
            voting_power: 5000u64.into(),
        }));
    assert!(!voters_hir
        .iter()
        .any(|hir| *hir == fred.as_voter_hir(REP_VOTING_GROUP)));
}
*/