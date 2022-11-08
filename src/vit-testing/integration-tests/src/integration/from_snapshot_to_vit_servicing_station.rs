use crate::Vote;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::sync::Arc;

use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::MainnetWallet;
use crate::common::{iapyx_from_mainnet, RepsVoterAssignerSource};
use assert_fs::TempDir;
use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::Threshold;
use chain_impl_mockchain::block::BlockDate;
use fraction::Fraction;
use jormungandr_automation::testing::time;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_tools::network::{MainnetNetworkBuilder, MainnetWalletStateBuilder};
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_lib::v0::endpoints::snapshot::RawSnapshotInput;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vit_servicing_station_tests::common::raw_snapshot::RawSnapshot;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::config::{Role, DIRECT_VOTING_GROUP};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;
use snapshot_lib::{
    voting_group::{DEFAULT_DIRECT_VOTER_GROUP, DEFAULT_REPRESENTATIVE_GROUP}};

#[test]
pub fn voters_with_at_least_one_vote() {
    let testing_directory = TempDir::new().unwrap().into_persistent();

    let stake = 10_000;

    let alice_wallet = MainnetWallet::new(stake);
    let bob_wallet = MainnetWallet::new(stake);
    let clarice_wallet = MainnetWallet::new(stake);

    let (db_sync, reps) = MainnetNetworkBuilder::default()
        .with(alice_wallet.as_direct_voter())
        .with(bob_wallet.as_direct_voter())
        .with(clarice_wallet.as_direct_voter())
        .build();

    let job_params = JobParameters::fund("fund9");
    let filter_result =
        mock::do_snapshot(&db_sync, job_params.clone(), &testing_directory);

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 30,
    };

    let config = ConfigBuilder::default().block0_initials(Block0Initials(vec![
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

    let (nodes, vit_station, wallet_proxy) = spawn_network(
        &mut controller,
        vit_parameters,
        network_params,
        &mut template_generator,
    )
    .unwrap();

    let mut alice = iapyx_from_mainnet(&alice_wallet, &wallet_proxy).unwrap();
    let mut bob = iapyx_from_mainnet(&bob_wallet, &wallet_proxy).unwrap();

    let registration = filter_result.registrations().clone();
    let raw_snapshot = RawSnapshot {
        tag: job_params.tag.unwrap(),
        content: RawSnapshotInput {
            snapshot: snapshot_lib::RawSnapshot::from(registration),
            update_timestamp: 0,
            min_stake_threshold: 450u64.into(),
            voting_power_cap: Fraction::from(1u64),
            direct_voters_group: Some(DEFAULT_DIRECT_VOTER_GROUP.to_string()),
            representatives_group: Some(DEFAULT_REPRESENTATIVE_GROUP.to_string()),
        },
    };

    println!("snap {:#?}", raw_snapshot);
    assert!(vit_station.check_running());
    println!("poposals {:?}", vit_station.proposals(DEFAULT_REPRESENTATIVE_GROUP));
    println!(" tages {:?}",vit_station.snapshot_tags());

    vit_station.put_raw_snapshot(&RawSnapshot::default()).unwrap();


}
