use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::RepsVoterAssignerSource;
use assert_fs::TempDir;
use chain_addr::Discrimination;
use fraction::Fraction;
use mainnet_lib::MainnetWallet;
use mainnet_lib::{MainnetNetworkBuilder, MainnetWalletStateBuilder};
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Block0Initials;
use vitup::config::ConfigBuilder;
use vitup::config::SnapshotInitials;
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn cip_36_support() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let voting_threshold = 1;
    let tag = Some("test".to_string());

    let job_param = JobParameters {
        slot_no: None,
        tag: tag.clone(),
    };

    let alice = MainnetWallet::new(1_000);
    let bob = MainnetWallet::new(1_000);
    let clarice = MainnetWallet::new(1_000);
    let dave = MainnetWallet::new(1_000);

    let (db_sync, reps) = MainnetNetworkBuilder::default()
        .with(alice.as_direct_voter())
        .with(bob.as_representative())
        .with(clarice.as_representative())
        .with(dave.as_delegator(vec![(&bob, 1u8), (&clarice, 1u8)]))
        .build(&testing_directory);

    let snapshot_result = mock::do_snapshot(&db_sync, job_param, &testing_directory).unwrap();

    let snapshot_filter = snapshot_result.filter(
        voting_threshold.into(),
        Fraction::new(1u64, 3u64),
        &reps.into_reps_voter_assigner(),
    );

    let config = ConfigBuilder::default()
        .voting_power(voting_threshold)
        .block0_initials(Block0Initials::new_from_external(
            snapshot_filter.to_voters_hirs(),
            Discrimination::Production,
        ))
        .snapshot_initials(SnapshotInitials::from_voters_hir(
            snapshot_filter.to_voters_hirs(),
            tag.unwrap_or_else(|| "".to_string()),
        ))
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();

    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();
    let (_nodes, _vit_station, _wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();
}
