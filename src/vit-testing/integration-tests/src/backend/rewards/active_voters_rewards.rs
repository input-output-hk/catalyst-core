use crate::Vote;

use crate::common::iapyx_from_mainnet;
use crate::common::MainnetWallet;
use assert_fs::TempDir;
use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::snapshot::RawSnapshot;
use catalyst_toolbox::snapshot::Snapshot;
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::testing::time;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn voters_with_at_least_one_vote() {
    let stake = 10_000;

    let alice_wallet = MainnetWallet::new(stake);
    let bob_wallet = MainnetWallet::new(stake);
    let clarice_wallet = MainnetWallet::new(stake);

    let raw_snapshot = vec![
        alice_wallet.as_voting_registration(),
        bob_wallet.as_voting_registration(),
        clarice_wallet.as_voting_registration(),
    ];

    let snapshot = Snapshot::from_raw_snapshot(RawSnapshot::from(raw_snapshot), 450.into());
    let testing_directory = TempDir::new().unwrap().into_persistent();

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

    let fund1_vote_plan = &controller.defined_vote_plans()[0];

    alice
        .vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    bob.vote_for(fund1_vote_plan.id(), 1, Vote::Yes as u8)
        .unwrap();

    bob.vote_for(fund1_vote_plan.id(), 0, Vote::Yes as u8)
        .unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 0,
    };
    time::wait_for_date(target_date.into(), nodes[0].rest());

    let block0 = &controller.settings().block0;
    let records = calc_voter_rewards(
        nodes[0].rest().account_votes_count().unwrap(),
        1,
        block0,
        snapshot,
        100u32.into(),
    )
    .unwrap();

    assert_eq!(
        records
            .iter()
            .find(|(x, _y)| **x == alice_wallet.reward_address())
            .unwrap()
            .1,
        &50u32.into()
    );

    assert_eq!(
        records
            .iter()
            .find(|(x, _y)| **x == bob_wallet.reward_address())
            .unwrap()
            .1,
        &50u32.into()
    );
}
