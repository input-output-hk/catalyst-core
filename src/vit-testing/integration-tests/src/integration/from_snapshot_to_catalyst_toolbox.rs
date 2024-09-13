/*
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::{CardanoWallet, RepsVoterAssignerSource};
use assert_fs::TempDir;
use fraction::Fraction;
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_lib::VoterHIR;
use snapshot_trigger_service::config::JobParameters;
use vitup::config::{DIRECT_VOTING_GROUP, REP_VOTING_GROUP};
*/
/* BROKEN TEST - Because VoterHIR has no `eq`
#[test]
pub fn cip36_mixed_delegation() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let stake = 10_000;

    let alice = CardanoWallet::new(stake);
    let bob = CardanoWallet::new(stake);
    let clarice = CardanoWallet::new(stake);

    let david = CardanoWallet::new(0);
    let edgar = CardanoWallet::new(0);
    let fred = CardanoWallet::new(0);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_delegator(vec![(&david, 1)]))
        .with(clarice.as_delegator(vec![(&david, 1), (&edgar, 1)]))
        .build();

    let voter_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter(
            450u64.into(),
            Fraction::from(1u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert_eq!(voter_hir.len(), 3);
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: alice.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: stake.into(),
        address: alice.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,
    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: david.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: (stake + stake / 2).into(),
        address: david.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,

    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: edgar.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: (stake / 2).into(),
        address: edgar.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,

    }));
    assert!(!voter_hir
        .iter()
        .any(|x| x.voting_key == fred.catalyst_public_key()));
}
*/

/* BROKEN TEST - Because VoterHIR has no `eq`
#[test]
pub fn voting_power_cap_for_reps() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice = CardanoWallet::new(1_000);
    let bob = CardanoWallet::new(1_000);
    let clarice = CardanoWallet::new(10_000);

    let david = CardanoWallet::new(0);
    let edgar = CardanoWallet::new(0);
    let fred = CardanoWallet::new(0);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_delegator(vec![(&david, 1)]))
        .with(bob.as_delegator(vec![(&edgar, 1)]))
        .with(clarice.as_delegator(vec![(&fred, 1)]))
        .build();

    let reps_circle_size = reps.len();

    let voter_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter(
            450u64.into(),
            Fraction::new(1u64, reps_circle_size as u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: fred.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
        address: fred.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,
    }));
}
*/

/* BROKEN TEST - Because VoterHIR has no `eq`
#[test]
pub fn voting_power_cap_for_direct() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice = CardanoWallet::new(10_000);
    let bob = CardanoWallet::new(10_000);
    let clarice = CardanoWallet::new(1_000);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_direct_voter())
        .with(clarice.as_direct_voter())
        .build();

    let voter_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter(
            450u64.into(),
            Fraction::new(1u64, 3u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: clarice.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
        address: david.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,
    }));
}
*/

/* BROKEN TEST - Because VoterHIR has no `eq`
#[test]
pub fn voting_power_cap_for_mix() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice = CardanoWallet::new(10_000);
    let bob = CardanoWallet::new(1_000);
    let clarice = CardanoWallet::new(10_000);

    let david = CardanoWallet::new(0);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_direct_voter())
        .with(clarice.as_delegator(vec![(&david, 1)]))
        .build();

    let voter_hir = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter_default(&reps)
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: alice.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
        address: alice.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,
    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: david.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
        address: david.catalyst_address(),
        underthreshold: false,
        overlimit: false,
        private_key: None,
    }));
}
*/
