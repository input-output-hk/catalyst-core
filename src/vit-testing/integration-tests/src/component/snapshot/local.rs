use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_lib::VoterHIR;
use snapshot_trigger_service::config::ConfigurationBuilder;
use snapshot_trigger_service::config::JobParameters;
use std::collections::HashSet;

const DIRECT_VOTING_GROUP: &str = "direct";
const REP_VOTING_GROUP: &str = "rep";

#[test]
pub fn mixed_registration_transactions() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;

    let alice_voter = MainnetWallet::new(stake);
    let bob_voter = MainnetWallet::new(stake);
    let clarice_voter = MainnetWallet::new(stake);

    let david_representative = MainnetWallet::new(500);
    let edgar_representative = MainnetWallet::new(1_000);
    let fred_representative = MainnetWallet::new(8_000);

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

    let voters_hir = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(JobParameters::fund("fund9"))
        .filter_default(&reps)
        .to_voters_hirs();

    assert!(voters_hir
        .iter()
        .any(|hir| *hir == alice_voter.as_voter_hir(DIRECT_VOTING_GROUP)));
    assert!(!voters_hir
        .iter()
        .any(|hir| *hir == bob_voter.as_voter_hir(DIRECT_VOTING_GROUP)));
    assert!(voters_hir.iter().any(|hir| *hir
        == VoterHIR {
            voting_key: david_representative.catalyst_public_key(),
            voting_group: REP_VOTING_GROUP.to_string(),
            voting_power: 15000u64.into(),
        }));
    assert!(voters_hir.iter().any(|hir| *hir
        == VoterHIR {
            voting_key: edgar_representative.catalyst_public_key(),
            voting_group: REP_VOTING_GROUP.to_string(),
            voting_power: 5000u64.into(),
        }));
    assert!(!voters_hir
        .iter()
        .any(|hir| *hir == fred_representative.as_voter_hir(REP_VOTING_GROUP)));
}
