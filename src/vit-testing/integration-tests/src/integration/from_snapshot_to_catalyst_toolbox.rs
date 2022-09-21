use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::{
    MainnetWallet, RepsVoterAssignerSource, DIRECT_VOTING_GROUP, REP_VOTING_GROUP,
};
use assert_fs::TempDir;
use fraction::Fraction;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_lib::VoterHIR;
use snapshot_trigger_service::config::ConfigurationBuilder;
use snapshot_trigger_service::config::JobParameters;
use std::collections::HashSet;

#[test]
pub fn cip36_mixed_delegation() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let stake = 10_000;

    let alice_voter = MainnetWallet::new(stake);
    let bob_voter = MainnetWallet::new(stake);
    let clarice_voter = MainnetWallet::new(stake);

    let david_representative = MainnetWallet::new(0);
    let edgar_representative = MainnetWallet::new(0);
    let fred_representative = MainnetWallet::new(0);

    let mut reps = HashSet::new();
    reps.insert(edgar_representative.catalyst_public_key());
    reps.insert(david_representative.catalyst_public_key());
    reps.insert(fred_representative.catalyst_public_key());

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    bob_voter
        .send_delegated_voting_registration(vec![(david_representative.catalyst_public_key(), 1)])
        .to(&mut mainnet_network)
        .unwrap();
    clarice_voter
        .send_delegated_voting_registration(vec![
            (david_representative.catalyst_public_key(), 1),
            (edgar_representative.catalyst_public_key(), 1),
        ])
        .to(&mut mainnet_network)
        .unwrap();

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let voter_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"))
        .filter(
            450u64.into(),
            Fraction::from(1u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert_eq!(voter_hir.len(), 3);
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: alice_voter.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: stake.into(),
    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: david_representative.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: (stake + stake / 2).into(),
    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: edgar_representative.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: (stake / 2).into(),
    }));
    assert!(!voter_hir
        .iter()
        .any(|x| x.voting_key == fred_representative.catalyst_public_key()));
}

#[test]
pub fn voting_power_cap_for_reps() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice_voter = MainnetWallet::new(1_000);
    let bob_voter = MainnetWallet::new(1_000);
    let clarice_voter = MainnetWallet::new(10_000);

    let david_representative = MainnetWallet::new(0);
    let edgar_representative = MainnetWallet::new(0);
    let fred_representative = MainnetWallet::new(0);

    let mut reps = HashSet::new();
    reps.insert(edgar_representative.catalyst_public_key());
    reps.insert(david_representative.catalyst_public_key());
    reps.insert(fred_representative.catalyst_public_key());

    let reps_circle_size = reps.len();

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_delegated_voting_registration(vec![(david_representative.catalyst_public_key(), 1)])
        .to(&mut mainnet_network)
        .unwrap();
    bob_voter
        .send_delegated_voting_registration(vec![(edgar_representative.catalyst_public_key(), 1)])
        .to(&mut mainnet_network)
        .unwrap();
    clarice_voter
        .send_delegated_voting_registration(vec![(fred_representative.catalyst_public_key(), 1)])
        .to(&mut mainnet_network)
        .unwrap();

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let voter_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"))
        .filter(
            450u64.into(),
            Fraction::new(1u64, reps_circle_size as u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: fred_representative.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
    }));
}

#[test]
pub fn voting_power_cap_for_direct() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice_voter = MainnetWallet::new(10_000);
    let bob_voter = MainnetWallet::new(10_000);
    let clarice_voter = MainnetWallet::new(1_000);

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    bob_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    clarice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let reps = HashSet::new();

    let voter_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"))
        .filter(
            450u64.into(),
            Fraction::new(1u64, 3u64),
            &reps.into_reps_voter_assigner(),
        )
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: clarice_voter.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
    }));
}

#[test]
pub fn voting_power_cap_for_mix() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let alice_voter = MainnetWallet::new(10_000);
    let bob_voter = MainnetWallet::new(1_000);
    let clarice_voter = MainnetWallet::new(10_000);

    let david_representative = MainnetWallet::new(0);

    let mut reps = HashSet::new();
    reps.insert(david_representative.catalyst_public_key());

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    bob_voter
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    clarice_voter
        .send_delegated_voting_registration(vec![(david_representative.catalyst_public_key(), 1)])
        .to(&mut mainnet_network)
        .unwrap();

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let voter_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"))
        .filter_default(&reps)
        .to_voters_hirs();

    assert!(voter_hir.contains(&VoterHIR {
        voting_key: alice_voter.catalyst_public_key(),
        voting_group: DIRECT_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
    }));
    assert!(voter_hir.contains(&VoterHIR {
        voting_key: david_representative.catalyst_public_key(),
        voting_group: REP_VOTING_GROUP.to_string(),
        voting_power: 1000.into(),
    }));
}
