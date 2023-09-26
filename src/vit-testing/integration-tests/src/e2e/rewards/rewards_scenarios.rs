use crate::Vote;
use std::collections::HashSet;

use crate::common::iapyx_from_mainnet;
use crate::common::mainnet_wallet_ext::MainnetWalletExtension;
use crate::common::snapshot::mock;
use crate::common::snapshot_filter::SnapshotFilterSource;
use crate::common::CardanoWallet;
use assert_fs::TempDir;
use catalyst_toolbox::rewards::dreps::calc_dreps_rewards;
use catalyst_toolbox::rewards::voters::calc_voter_rewards;
use catalyst_toolbox::rewards::{Threshold, VoteCount};
use chain_impl_mockchain::block::BlockDate;
use jormungandr_automation::testing::time;
use jormungandr_lib::crypto::account::Identifier;
use mainnet_lib::{wallet_state::MainnetWalletStateBuilder, MainnetNetworkBuilder};
use snapshot_lib::registration::RewardAddress;
use snapshot_lib::VotingGroup;
use snapshot_trigger_service::config::JobParameters;
use vit_servicing_station_tests::common::data::ArbitraryValidVotingTemplateGenerator;
use vitup::config::Role;
use vitup::config::VoteBlockchainTime;
use vitup::config::{Block0Initials, ConfigBuilder};
use vitup::testing::spawn_network;
use vitup::testing::vitup_setup;

const CHALLENGES_COUNT: usize = 1;
const PROPOSALS_COUNT: u32 = 4;
const VOTE_THRESHOLD_PER_CHALLENGE: u32 = 1;
const VOTE_THRESHOLD_PER_VOTER: usize = 1;
const TOTAL_REWARD: u32 = 100;
const EXPECTED_REWARD: u32 = 25;
const STAKE: u64 = 1000;
const STAKE_X2: u64 = 2000;

#[test] // 2 direct voter, 2 dreps  all with equal amount of voting power
pub fn mixed_rewards_happy_path() {
    let rep_voting_group: VotingGroup = "rep".to_string();

    let testing_directory = TempDir::new().unwrap().into_persistent();

    let drep1_wallet = CardanoWallet::new(STAKE);
    let drep2_wallet = CardanoWallet::new(STAKE);

    let alice_wallet = CardanoWallet::new(STAKE);
    let bob_wallet = CardanoWallet::new(STAKE);
    let clarice_wallet = CardanoWallet::new(STAKE);
    let john_wallet = CardanoWallet::new(STAKE);

    let emma_wallet = CardanoWallet::new(STAKE_X2);
    let jim_wallet = CardanoWallet::new(STAKE_X2);

    let (db_sync, _node, reps) = MainnetNetworkBuilder::default()
        .with(drep1_wallet.as_representative())
        .with(drep2_wallet.as_representative())
        .with(alice_wallet.as_delegator(vec![(&drep1_wallet, STAKE as u8)]))
        .with(bob_wallet.as_delegator(vec![(&drep1_wallet, STAKE as u8)]))
        .with(clarice_wallet.as_delegator(vec![(&drep2_wallet, STAKE as u8)]))
        .with(john_wallet.as_delegator(vec![(&drep2_wallet, STAKE as u8)]))
        .with(emma_wallet.as_direct_voter())
        .with(jim_wallet.as_direct_voter())
        .build();

    let snapshot = mock::do_snapshot(&db_sync, JobParameters::fund("fund10"), &testing_directory)
        .unwrap()
        .filter_default(&HashSet::new());

    let mut snapshot_doctored_copy = Vec::new();
    let mut drep_hirs = Vec::new();

    for voter_info in snapshot.snapshot().to_full_snapshot_info() {
        if reps.contains(&voter_info.hir.voting_key) {
            let mut copy = voter_info;
            copy.hir.voting_group = rep_voting_group.clone();
            snapshot_doctored_copy.push(copy.clone());
            drep_hirs.push(copy.clone().hir.voting_key)
        } else {
            snapshot_doctored_copy.push(voter_info);
        }
    }

    let vote_timing = VoteBlockchainTime {
        vote_start: 0,
        tally_start: 1,
        tally_end: 2,
        slots_per_epoch: 15,
    };

    let config = ConfigBuilder::default()
        .block0_initials(Block0Initials(vec![
            drep1_wallet.as_initial_entry(),
            drep2_wallet.as_initial_entry(),
            emma_wallet.as_initial_entry(),
            jim_wallet.as_initial_entry(),
        ]))
        .vote_timing(vote_timing.into())
        .slot_duration_in_seconds(2)
        .challenges_count(CHALLENGES_COUNT)
        .proposals_count(PROPOSALS_COUNT)
        .voting_power(1000000)
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

    let mut drep1 = iapyx_from_mainnet(&drep1_wallet, &wallet_proxy).unwrap();
    let mut drep2 = iapyx_from_mainnet(&drep2_wallet, &wallet_proxy).unwrap();
    let mut emma = iapyx_from_mainnet(&emma_wallet, &wallet_proxy).unwrap();
    let mut jim = iapyx_from_mainnet(&jim_wallet, &wallet_proxy).unwrap();

    let voteplan_alias = format!(
        "{}-{}",
        config.data.current_fund.fund_info.fund_name,
        Role::Voter
    );
    let vote_plan = controller.defined_vote_plan(&voteplan_alias).unwrap();

    drep1.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    drep1.vote_for(&vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    drep1.vote_for(&vote_plan.id(), 2, Vote::Yes as u8).unwrap();

    drep1.vote_for(&vote_plan.id(), 3, Vote::Yes as u8).unwrap();

    drep2.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    drep2.vote_for(&vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    drep2.vote_for(&vote_plan.id(), 2, Vote::Yes as u8).unwrap();

    drep2.vote_for(&vote_plan.id(), 3, Vote::Yes as u8).unwrap();

    emma.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    emma.vote_for(&vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    emma.vote_for(&vote_plan.id(), 2, Vote::Yes as u8).unwrap();

    emma.vote_for(&vote_plan.id(), 3, Vote::Yes as u8).unwrap();

    jim.vote_for(&vote_plan.id(), 0, Vote::Yes as u8).unwrap();

    jim.vote_for(&vote_plan.id(), 1, Vote::Yes as u8).unwrap();

    jim.vote_for(&vote_plan.id(), 2, Vote::Yes as u8).unwrap();

    jim.vote_for(&vote_plan.id(), 3, Vote::Yes as u8).unwrap();

    let target_date = BlockDate {
        epoch: 1,
        slot_id: 0,
    };

    time::wait_for_date(target_date.into(), nodes[0].rest());

    let proposals = vit_station.proposals(&rep_voting_group).unwrap();

    let account_votes_count: VoteCount = nodes[0]
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

    let drep_records = calc_dreps_rewards(
        snapshot_doctored_copy.clone(),
        account_votes_count.clone(),
        rep_voting_group as VotingGroup,
        4,
        Threshold::new(
            VOTE_THRESHOLD_PER_VOTER,
            vit_station
                .challenges()
                .unwrap()
                .iter()
                .map(|x| (x.id, VOTE_THRESHOLD_PER_CHALLENGE as usize))
                .collect(),
            proposals.clone().into_iter().map(Into::into).collect(),
        )
        .unwrap(),
        TOTAL_REWARD.into(),
    )
    .unwrap();

    let direct_records = calc_voter_rewards(
        account_votes_count,
        snapshot_doctored_copy.clone(),
        Threshold::new(
            VOTE_THRESHOLD_PER_VOTER,
            vit_station
                .challenges()
                .unwrap()
                .iter()
                .map(|x| (x.id, VOTE_THRESHOLD_PER_CHALLENGE as usize))
                .collect(),
            proposals.into_iter().map(Into::into).collect(),
        )
        .unwrap(),
        TOTAL_REWARD.into(),
    )
    .unwrap();

    for drep in drep_hirs {
        assert_eq!(
            drep_records.iter().find(|(x, _y)| **x == drep).unwrap().1,
            &EXPECTED_REWARD.into()
        );
        println!("Assertion passed for drep");
    }

    assert_eq!(
        direct_records
            .iter()
            .find(|(x, _y)| **x
                == RewardAddress(emma_wallet.reward_address().to_address().to_hex()))
            .unwrap()
            .1,
        &EXPECTED_REWARD.into()
    );

    println!("Assertion passed for emma");

    assert_eq!(
        direct_records
            .iter()
            .find(|(x, _y)| **x == RewardAddress(jim_wallet.reward_address().to_address().to_hex()))
            .unwrap()
            .1,
        &EXPECTED_REWARD.into()
    );

    println!("Assertion passed for jim");
}
