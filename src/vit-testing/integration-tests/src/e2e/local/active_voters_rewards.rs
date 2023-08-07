use crate::Vote;
use std::collections::HashSet;

use crate::common::iapyx_from_mainnet;
use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::CardanoWallet;
use assert_fs::TempDir;
use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::Threshold;
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::testing::time;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_lib::registration::RewardAddress;
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::config::{Role, DIRECT_VOTING_GROUP};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

#[test]
pub fn voters_with_at_least_one_vote() {
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

    let snapshot = mock::do_snapshot(&db_sync, JobParameters::fund("fund9"), &testing_directory)
        .unwrap()
        .filter_default(&HashSet::new());

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

    let (nodes, vit_station, wallet_proxy) = spawn_network(
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

    alice.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    bob.vote_for(&vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    bob.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 0,
    };
    time::wait_for_date(target_date.into(), nodes[0].rest());

    let proposals = vit_station.proposals(DIRECT_VOTING_GROUP).unwrap();

    let account_votes_count = nodes[0]
        .rest()
        .account_votes_all()
        .unwrap()
        .iter()
        .map(|(id, account_votes)| {
            let votes = account_votes
                .iter()
                .map(|av| {
                    let vps = nodes[0].rest().vote_plan_statuses().unwrap();

                    let mut proposals = HashSet::new();
                    for vote_index in av.votes.iter() {
                        for status in &vps {
                            if status.id == av.vote_plan_id {
                                proposals.insert(
                                    status
                                        .proposals
                                        .iter()
                                        .find(|p| p.index == *vote_index)
                                        .unwrap()
                                        .proposal_id,
                                );
                            }
                        }
                    }
                    proposals
                })
                .fold(HashSet::new(), |mut acc, items| {
                    for item in items {
                        acc.insert(item);
                    }
                    acc
                });
            (Identifier::from_hex(id).unwrap(), votes)
        })
        .collect();

    let records = calc_voter_rewards(
        account_votes_count,
        snapshot.snapshot().to_full_snapshot_info(),
        Threshold::new(
            1_000_000,
            vit_station
                .challenges()
                .unwrap()
                .iter()
                .map(|x| (x.id, x.proposers_rewards as usize))
                .collect(),
            proposals.into_iter().map(Into::into).collect(),
        )
        .unwrap(),
        1_000_000u32.into(),
    )
    .unwrap();

    println!("{:?}", records);

    assert_eq!(
        records
            .iter()
            .find(
                |(x, _y)| **x == RewardAddress(alice_wallet.reward_address().to_address().to_hex())
            )
            .unwrap()
            .1,
        &50u32.into()
    );

    assert_eq!(
        records
            .iter()
            .find(|(x, _y)| **x == RewardAddress(bob_wallet.reward_address().to_address().to_hex()))
            .unwrap()
            .1,
        &50u32.into()
    );
}
