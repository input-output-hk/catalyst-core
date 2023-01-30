use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::mock;
use crate::common::CardanoWallet;
use assert_fs::TempDir;
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_lib::SnapshotInfo;
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::raw_snapshot::RawSnapshotBuilder;
use vit_servicing_station_tests::common::snapshot::VotingPower;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn put_raw_snapshot() {
    let testing_directory = TempDir::new().unwrap().into_persistent();
    let stake = 10_000;
    let alice_wallet = CardanoWallet::new(stake);
    let bob_wallet = CardanoWallet::new(stake);
    let clarice_wallet = CardanoWallet::new(stake);

    let (db_sync, _node, _reps) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_direct_voter())
        .with(bob_wallet.as_direct_voter())
        .with(clarice_wallet.as_direct_voter())
        .build();

    let job_params = JobParameters::fund("fund9");
    let snapshot_result =
        mock::do_snapshot(&db_sync, job_params.clone(), &testing_directory).unwrap();

    let vote_timing = VoteBlockchainTime {
        vote_start: 1,
        tally_start: 2,
        tally_end: 3,
        slots_per_epoch: 30,
    };

    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![
            alice_wallet.as_initial_entry(),
            bob_wallet.as_initial_entry(),
            clarice_wallet.as_initial_entry(),
        ]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .proposals_count(3)
        .voting_power(100)
        .private(false)
        .build();

    let mut template_generator = ArbitraryValidVotingTemplateGenerator::new();
    let (mut controller, vit_parameters, network_params) =
        vitup_setup(&config, testing_directory.path().to_path_buf()).unwrap();

    let (_nodes, vit_station, _wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let registrations = snapshot_result.registrations().clone();

    let raw_snapshot = RawSnapshotBuilder::default()
        .with_voting_registrations(registrations)
        .with_tag(job_params.tag.as_ref().unwrap())
        .build();

    assert!(vit_station.check_running());

    vit_station.put_raw_snapshot(&raw_snapshot).unwrap();

    assert_eq!(
        vec![job_params.tag.unwrap()],
        vit_station.snapshot_tags().unwrap(),
        "expected tags vs tags taken from REST API"
    );

    let snapshot_infos: Vec<SnapshotInfo> = raw_snapshot.clone().try_into().unwrap();

    for snapshot_info in snapshot_infos.iter() {
        let voting_power = VotingPower::from(snapshot_info.clone());
        let voter_info = vit_station
            .voter_info(&raw_snapshot.tag, &snapshot_info.hir.voting_key.to_hex())
            .unwrap();
        assert_eq!(
            vec![voting_power],
            voter_info.voter_info,
            "wrong data for entry: {:?}",
            snapshot_info
        );
        assert_eq!(
            raw_snapshot.content.update_timestamp, voter_info.last_updated,
            "wrong timestamp for entry: {:?}",
            snapshot_info
        );
        for contribution in snapshot_info.contributions.iter() {
            let delegator_info = vit_station
                .delegator_info(&raw_snapshot.tag, &contribution.stake_public_key)
                .unwrap();
            assert!(
                delegator_info
                    .dreps
                    .contains(&snapshot_info.hir.voting_key.to_hex()),
                "wrong data for entry: {:?}",
                snapshot_info
            );
            assert!(
                delegator_info
                    .voting_groups
                    .contains(&snapshot_info.hir.voting_group),
                "wrong data for entry: {:?}",
                snapshot_info
            );
            assert_eq!(
                delegator_info.last_updated, raw_snapshot.content.update_timestamp,
                "wrong data for entry: {:?}",
                snapshot_info
            );
        }
    }
}
