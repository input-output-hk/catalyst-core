use crate::Vote;

use crate::common::iapyx_from_mainnet;
use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::SnapshotServiceStarter;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::snapshot::snapshot_test_api::DummyAssigner;
use chain_impl_mockchain::block::BlockDate;
use fraction::Fraction;
use jormungandr_automation::testing::time;
use mainnet_tools::db_sync::DbSyncInstance;
use mainnet_tools::network::MainnetNetwork;
use mainnet_tools::voting_tools::VotingToolsMock;
use snapshot_trigger_service::config::ConfigurationBuilder;
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Role;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn voters_with_at_least_one_vote() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let stake = 10_000;

    let alice_wallet = MainnetWallet::new(stake);
    let bob_wallet = MainnetWallet::new(stake);
    let clarice_wallet = MainnetWallet::new(stake);

    let mut mainnet_network = MainnetNetwork::default();
    let mut db_sync_instance = DbSyncInstance::default();

    mainnet_network.sync_with(&mut db_sync_instance);

    alice_wallet
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    bob_wallet
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();
    clarice_wallet
        .send_direct_voting_registration()
        .to(&mut mainnet_network)
        .unwrap();

    let voting_tools =
        VotingToolsMock::default().connect_to_db_sync(&db_sync_instance, &testing_directory);

    let configuration = ConfigurationBuilder::default()
        .with_voting_tools_params(voting_tools.into())
        .with_tmp_result_dir(&testing_directory)
        .build();

    let snapshot = SnapshotServiceStarter::default()
        .with_configuration(configuration)
        .start_on_available_port(&testing_directory)
        .unwrap()
        .snapshot(
            JobParameters::fund("fund9"),
            450u64,
            Fraction::from(1u64),
            &DummyAssigner,
        );

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
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

    let (nodes, _vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let mut alice = iapyx_from_mainnet(&alice_wallet, &wallet_proxy).unwrap();
    let mut bob = iapyx_from_mainnet(&bob_wallet, &wallet_proxy).unwrap();

    let voteplan_alias = format!(
        "{}-{}",
        config.data.current_fund.fund_info.fund_name,
        Role::Voter
    );
    let vote_plan = controller.defined_vote_plan(&voteplan_alias).unwrap();

    alice.vote_for(vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    bob.vote_for(vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    bob.vote_for(vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 0,
    };
    time::wait_for_date(target_date.into(), nodes[0].rest());

    let account_votes_count = nodes[0]
        .rest()
        .account_votes_all()
        .unwrap()
        .iter()
        .map(|(a, account_votes)| {
            (
                a.clone(),
                account_votes
                    .iter()
                    .map(|av| av.votes.clone())
                    .fold(0u64, |sum, votes| sum + votes.len() as u64),
            )
        })
        .collect();

    println!("{:?}", account_votes_count);

    let records = calc_voter_rewards(
        account_votes_count,
        1,
        snapshot.to_full_snapshot_info(),
        100u32.into(),
    )
    .unwrap();

    println!("{:?}", records);

    assert_eq!(
        records
            .iter()
            .find(|(x, _y)| **x == alice_wallet.reward_address_as_bech32())
            .unwrap()
            .1,
        &50u32.into()
    );

    assert_eq!(
        records
            .iter()
            .find(|(x, _y)| **x == bob_wallet.reward_address_as_bech32())
            .unwrap()
            .1,
        &50u32.into()
    );
}
